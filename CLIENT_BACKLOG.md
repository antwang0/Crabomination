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

## Right-click hints, conspire, your decks (2026-09-30) — shipped

Read off layout-harness screenshots: `--hover-card NAME` now hovers a
hand card too, the fixture hand carries a kicker card and an MDFC, and
`--saved-decks` opens "Your Decks" on the menu.

- ✅ **A hand card says what right-click does with it.** Right-click was
  a priority cascade in `handle_game_input` — pitch, alternative cost,
  squad / replicate / multikicker, Spree, split, gift, Omen, splice,
  convoke, kicker, flip, else the play menu — with nothing on the card to
  say which branch it would take. The cascade is one pure function now
  (`hand_menu::right_click`) that the input handler matches on and a chip
  reads (`hand_chips`, which absorbed the FREE chip): a 🖰 at the card's
  top-left corner, spelled out — "🖰 Pay kicker", "🖰 Flip to Shatterskull,
  the Hammer Pass", "🖰 Alt. cost: …" — while the card is hovered or picked
  by the keyboard cursor. A hovered permanent of yours with an ability
  menu reads "🖰 Abilities". It hides while right-click belongs to
  something else (targeting, attack or block picking, a decision, no
  priority), and a card whose menu would be just "Cast" gets none.
- ✅ **Conspire casts from the client** (CR 702.78). Right-clicking a
  conspirable hand card opens the helper picker over your untapped
  creatures that share a colour with it; Cast waits for exactly two ("Pick
  1 more") and submits `CastSpellConspire`, arming the targeting cursor
  first for a targeted spell. Closes "Conspire cast UI (follow-up)".
- ✅ **Imported decks are kept, and the menu lists them.** Every list that
  imports From File or From Clipboard is saved as text under
  `<config>/crabomination/decks/` (`saved_decks`), named for the name an
  Arena export's `About` header gives it, its commanders, the file it came
  from, or its most-played spell — once, however often the same cards come
  back. "Your Decks" lists them: a click plays one in the selected format
  (or opens the import report saying why it can't), Delete asks once. The
  Commander deck picker lists your legal Commander lists above the stock
  decks, for any seat, bots included (`DeckChoice::Saved`); one deleted
  since is dealt as Random. A list dropped into the folder by hand shows up
  the same way. The decklist parser skips Arena's `About` / `Name` header,
  which it had read as two unknown cards.
- ✅ **Engine: a card you'd tap lands for is castable.** Found through the
  chips. A human seat (`manual_mana`) whose cost has more than one set of
  sources to pay it gets `ManualTapRequired` back from the cast — pick what
  taps — and the affordance probe read that as a refusal, so the castable
  border and every alt-cast set dropped off any card with two ways to pay:
  in the harness fixture, Lightning Bolt, Burst Lightning and Shivan Dragon
  over two Mountains and a Stomping Ground. `accept_on` re-probes such a
  bounce with the engine tapping (bot seats never take the branch).
  Regression: `core_rules::game::castable_hand_cards_counts_a_cast_the_player_must_tap_for`.

## Rules text in decision modals, big hands, hover delay (2026-09-30) — shipped

Read off layout-harness screenshots: `--decision scry|search|discard`
opens that modal client-side over the fixture, `--hover-card NAME` then
hovers its card of that name, and `--hand N` deals the viewer N cards.

- ✅ **A card in a decision modal reads like one on the table.** Scry,
  search, discard / card-pick, trigger order and damage order/assignment
  tiles were 180 px art with no rules text, and the Alt peek only fired on
  3-D cards. Every tile now carries `ui_card_hover::UiCardHover` (name and
  id, not just an art path), and the one preview system
  (`ui::hover_card_preview`) serves both kinds: art plus the Oracle panel,
  a UI card winning over the table card under it. Alt over a modal card
  opens the large peek. It sits beside the modal's panel
  (`PreviewAnchor`), not beside the tile, where it covered the next card
  in the row. An ineligible search result previews too (it says why it is
  greyed out). The stack panel's rows, the log's lines and the graveyard /
  exile browser tiles gain the rules text the same way.
- ✅ **The preview waits for the pointer to settle, and fades in.** It
  opens after 300 ms on one card (`HOVER_DWELL`) and fades in over
  120 ms; moving straight to the next card, or back within a quarter
  second, swaps it at once, so reading along a row isn't slowed.
  `focus::fade_subtree` is the fade the card overlays already used, now
  shared and taking images too (an overlay's own images used to stay lit
  while the rest dimmed).
- ✅ **A big hand keeps the seven-card fan.** Spacing already stopped
  growing past seven, but the droop and tilt still grew by slot: a 15-card
  hand's end cards sank 2.1 units and turned 24°, below the window. They
  follow the card's place along the fan now (`layout::fan_arc`), so a big
  hand is the same arc, denser. Opponents' face-down pod fans too.

## Card art cache and client dependencies (2026-09-30) — shipped

- ✅ **Card art is cached as Scryfall's `large` JPEG, not its PNG.** The
  PNGs (745 × 1040, ~1.2 MB each) were 28 GB for the 22,193 images of the
  full catalog; `large` (672 × 936) is ~120 KB, a tenth of the disk and of
  the first-run download. The biggest a card is drawn is the 230 px hover
  preview, and a 2x crop of the rules text reads the same in both. The
  transparent PNG corners never showed on the table (the card mesh is a
  rounded rectangle); Scryfall's JPEGs fill them black.
  `scryfall::convert_png_art` converts an existing PNG cache once, in
  place, on the prefetch thread (scaled to 672 wide at quality 85 — 15 % of
  the PNG over a 40-card sample, full size at 90 was 26 % and read no
  better — with no re-download),
  renames the negative-cache entries, and hot-swaps each converted image
  in; the menu line reads "Converting cached card art". A missing `.jpg`
  still gets a drawn placeholder, served as JPEG because Bevy picks the
  decoder by extension; drawn proxy faces keep their `.png` paths.
- ✅ **The client compiles 68 fewer crates.** `image` and `imageproc` ran
  on default features (every codec, the rav1e AV1 encoder, rayon, FFT) for
  a PNG encode; Bevy carried glTF, scenes and `sysinfo_plugin`, none of
  them used. Cold `cargo build -p crabomination_client`: 597 -> 529 units.

## Two commanders in the command zone (2026-09-30) — shipped

Read off layout-harness screenshots (`--layout-fixture 2|3|4 --partners`,
which seats the pod's two-commander decks); card sizes from
`framing::tests::budget`.

- ✅ **A Partner pair (or a commander and its Background) shows both
  cards.** Every slot of a seat's command zone sat on one spot, a card's
  thickness apart, so the second commander hid the first and their cost
  chips overlapped. The zone is a lane now (`card::layout::command_lane`):
  the second card shows half its length past the first — name bar and art
  — toward the table centre in a pod; a fuller zone (schemes, conspiracies)
  shares the same reach. A whole card beside it ran under the HUD's side
  columns from a pod's side seats and cost 99 → 94 px viewer cards at
  1920x1080; half a card leaves every framing budget unchanged.
- ✅ **1v1 Commander's zone moved to the right-hand pile strip**, level
  with the graveyard across the board, clear of exile. It sat at x −11,
  inside the board, under the row's end card and the graveyard pile.
- ✅ Each cost chip sits past the end of its card that shows — the top of
  one its partner covers — and past a far seat's card rather than on it
  (a far card's bottom faces up the screen). A partner cast from the zone
  moves the other into the first slot.

## Token faces, lighter textures, nested rings (2026-09-29) — shipped

Read off layout-harness screenshots (`--tokens`, duel and pod; `--hover-card`);
memory from `/proc` RSS and `nvidia-smi`, three runs a side on the pod fixture.

- ✅ **Tokens without art get a drawn face** (`card::proxy`): a frame in the
  token's colours, its name on a dark bar, a big initial in the art box (so
  Goblins and Treasures differ at a glance), "Token Creature — Goblin", its
  keywords and a P/T box where the P/T badge sits. The facts are encoded
  into a `cards/proxy_….png` path and drawn by the placeholder reader, so
  the 3-D face, the hover preview and the Alt popup all show it and
  identical tokens share one texture. Any other art-less card's placeholder
  is the same frame with just its name. A token with art on disk keeps it.
- ✅ **Token art is fetched by exact name.** The prefetch searched
  `is:token t:<name>` and took the first hit, which is often a *named*
  token: "Soldier" was Ajani's Pridemate (a 2/2 Cat Soldier) and stood in
  for every 1/1 Soldier. It is `is:token !"<name>"` now, first single-faced
  printing; the old files are purged once (marker
  `cards/.token_art_by_exact_name`) and refetched.
- ✅ **Card textures reach the GPU once, with mips, and leave the CPU**
  (`card::mipmap::generate_card_mipmaps`): a new card image is held from
  the render world while its chain builds off-thread, then handed over as
  `RENDER_WORLD` only. Pod fixture: RSS 1486-1554 → 1316-1372 MiB, GPU
  1126 → 814 MiB over idle. No un-mipped first frames.
- ✅ **Highlight rings nest**: hover and dying rings are wider than the
  state rings they sit under, so a hovered castable (or activatable) card
  shows both colours; one width for all let the top ring hide the rest.
- ✅ The glow sorts after every other transparent mesh (its one mesh's
  centre wandered with the lights, so light over an arrow swapped order
  frame to frame). The felt, seat tints and prints and the eliminated
  shroud are `Pickable::IGNORE`. The hover preview builds its notes once
  per card and view instead of every frame. High and Ultra drop SMAA: over
  4x MSAA it took 10 % of card text's edge energy for no visible edge gain.

## Sharper cards, true colour, fitted shadows, idle frame pacing (2026-09-29) — shipped

Read off layout-harness screenshots at Low and High (duel, pod, `--hover-card`,
`--zoom-card`, `--stack`, `--impacts`, `--combat`), before and after; the
colour numbers are Lab statistics of Walking Ballista at the zoom pose against
its Scryfall image; frame rates from `CRAB_PERF=1`.

- ✅ **Card art shows as authored** (`SCENE_TONEMAPPING` in `main.rs`: no
  tonemapper). The source reads text box L* 82.5, art chroma 5.2; the old
  `TonyMcMapface` + 0.9 post-saturation drew 71.4 / 3.9 — the white text box
  eleven points grey, a quarter of the art's colour gone — and every filmic
  tonemapper Bevy ships drew the text box at 69-74. None draws 86.6 / 4.6.
  The lit felt and chips sit inside `[0, 1]`; glows driven past it clip toward
  their hue, and bloom still reads them where it is on.
- ✅ **Sharper table cards** (`MipBias(-0.5)` on the camera). A creature is
  ~125 px of a 745 px image at 1080p, so it sampled mip ~2.6 (93 texels wide,
  coarser than the pixels it covers); the 16x anisotropic filter barely
  engages at the table's 66°.
- ✅ **Shadows fitted to the table** (`systems::shadows`). Bevy's default
  four cascades out to 39.2 spent two on the air in front of the camera and
  cut off before the opponent's side (the duel's table spans view depths
  ~30-43; a pod's further). Two cascades are refit from the live camera. The
  felt, seat tints, zone prints, highlight rings, counter chips, the
  eliminated seat's shroud, three of four library-pile cards and the stack
  lane's cards no longer cast; with nothing flat in the map the biases drop
  (0.02 / 1.8 → 0.005 / 0.4), and a resting card's shadow meets its edge
  instead of a dark halo. Shadow maps: Low 512 → 1024, Ultra 8192 → 4096;
  VRAM at Ultra 1 GiB → 128 MiB.
- ✅ **The frame loop idles** (`systems::frame_pacing`). It rendered at the
  display's refresh rate forever; now a frame follows at once only when
  something changed (components, a material fade, art arriving, a short
  arrow animation), ambient light (the seat glow's breathing, arrow flow)
  ticks at ~30 fps, and a still table waits 250 ms (1 s unfocused — measured
  1.0 fps). Server messages wake it through a relay in front of `NetInbox`.
  Animations step by `animate::anim_dt` (capped at 1/30 s) so the frame that
  ends a wait doesn't jump one; `Time` stays true for the clocks.
  `CRAB_CONTINUOUS=1` restores continuous rendering; a harness screenshot run
  always renders continuously.
  - What held the loop awake, now written only when it changes (found with
    `CRAB_PERF=changes`): the hover lift (every card and library-pile card's
    `Transform`, every frame), pile positions and visibility, the camera's
    ease (never landed), the phase chart (12 texts re-shaped per frame), the
    P/T, keyword, counter, token, combat, lock, regen, agenda, free-cast and
    commander overlays and seat plates (`theme::place_overlay`), the hover
    preview, the attack-all panel, the HUD panel borders, the hint chip (its
    compare was defeated by a `Mut` → `&mut` coercion) and the hover tilt
    (untilt and retilt every frame).
- ✅ **Mipmaps off the main thread** (`card::mipmap`): built on the async
  compute pool (a stall per image in the `play` build, whose client crate is
  unoptimized), without the two full-image copies, and the sRGB encode is a
  table search instead of a `powf` per channel.
- ✅ **The game log adds rows** (`LogRow`, `LogEntry::seq`): it respawned all
  200 rows on every change — several times a second in a bot's turn.
- ✅ The graveyard pile keeps its material while its top card is the same
  (a new material per view); counter chips are `Pickable::IGNORE` (a chip
  took the pointer and its card un-hovered — not checked with a real mouse).
- ✅ Default quality is Medium (Low showed none of the post-processing);
  the unread `shadow_map_size` / `smaa_preset` settings and the embedded,
  never-loaded `models/woodtable_1.glb` are gone. `CRAB_PERF=1` shows a
  frame-time readout.

## In-game display pass (2026-09-27) — shipped

Read off layout-harness screenshots (1920x1080 and 1280x720, duel and
four-seat pod, on a 1.45x display). Before/after shots were compared by
eye; `framing::tests::budget` gates the table sizes.

- ✅ **A pod's boards say whose they are.** Each seat's deck pile carries a
  plate with its name and life (`table_tint::sync_seat_name_plates`); the
  seat tints told four boards apart but left you matching them to the
  roster in the corner. Pods only — a duel's far board needs no name.
- ✅ **Keyword strips sit on the top edge of the card as you see it**
  (`keyword_label`), whichever way the card faces. They hung from the
  card's own top edge, so an opponent's hung under their cards, a tapped
  card's beside it, and two pod boards facing each other printed theirs
  over each other along the seam ("F Fly g"). Superseded the same day:
  they sit on the P/T line ("Counters and card overlays", below).
- ✅ **Condition chips show only when a card of yours checks them.** Coven,
  threshold, metalcraft, ferocious, hellbent, formidable, descend, void,
  corrupted and crime were lit whenever met — a board of three creatures
  lit three of them in a game with no payoff for any. The test is a card in
  your hand, on your side or in your command zone whose definition names
  the condition (`player_stats::Condition`, from the definition's `Debug`
  text, memoised per name). The per-step strip ("UN UP DR M1 …") under the
  turn line is gone — the phase chart below it says the same — and the turn
  line names the step as the chart does ("Main 1", not `PreCombatMain`).
- ✅ **The prompt line moved under the action buttons**, as MTGO and XMage
  place theirs; it sat bottom-centre over the hand it was asking you to play
  from. `framing::hud_rects` reserves four lines of it: 47 → 46 px for a pod
  at 1280x720, nothing at 1920x1080 or larger.
- ✅ **Your chip row wraps short of the opponent panel**
  (`fit_player_hud_width`), at `framing::player_panel_width` — the width
  the camera fit already assumed — or sooner beside a pod's wider roster.
  At 1280x720 it ran under the opponent panel and covered its counts.
  - ⚠ **Cap the wrapping row, not the absolute panel.** With `max_width` on
    the panel, Taffy wrapped the row but sized the panel for one line, and
    the second hung below the border.
  - ⚠ `PlayerHudPanel` is on every seat's chip row, not only the viewer's
    panel, so `single()` on it fails whenever there is an opponent.
- ✅ **Mouse-wheel scrolling never worked** (found on the way). `bevy_ui`
  0.19 nodes carry `UiGlobalTransform`, not `GlobalTransform`, so
  `scroll::handle_scroll`'s query — and the animation-speed slider's —
  matched nothing: the log, the zone browsers and the library search sat
  clipped, as they had before the 2026-09-12 fix below claimed them.
  `scroll::tests::a_wheel_detent_scrolls_the_panel_under_the_cursor` fails
  on the old query.
  - ⚠ **`ComputedNode::size()` and `UiGlobalTransform` are physical
    pixels; the cursor and `Val::Px` are logical.** The game log's
    placement read the opponent panel's height as logical, parking the log
    45 % of that height too low on a 1.45x display.

## UI size and the menu (2026-09-27) — shipped

- ✅ **UI size** — Settings (and the in-game Esc menu) cycle Auto / 80 % /
  90 % / 100 % / 115 % / 130 % / 150 % / 175 % / 200 %, persisted as
  `graphics.ui_size` (0 = Auto) and applied through Bevy's `UiScale`
  (`theme::ui_scale_for`). Auto is 100 % up to a 1080-px-high window and
  grows with the height to 200 % at 2160, so a 4K window lays out as 1080p
  doubled (`framing::tests::budget`: 259 / 199 px cards, exactly twice
  1080p's). Any size is capped where the HUD would outgrow the window
  (`theme::MIN_UI_VIEWPORT`, 1024x640 UI px): 130 % of 1280x720 ran the
  prompt off the bottom, and is drawn at 112.5 %, which the row says.
  Layout harness: `--ui-size PERCENT`.
  - ⚠ **`UiScale` multiplies every `Val::Px`; a camera projection and the
    cursor are window px.** Every node placed over a card or at the
    cursor divides the scale back out — `theme::project_to_ui` for the
    twelve overlay modules (keyword strips, P/T and the other badges,
    counter labels, name plates, damage numerals, …), and the two hover
    previews, the ability and hand menus and the draft tooltip at their
    use. This is why `UiScale` had been pinned at 1.0: at any other value
    each of them lands that factor further from the window's corner.
  - ⚠ `ComputedNode::inverse_scale_factor` includes `UiScale`, so
    `size() * inverse_scale_factor()` is already UI px.
  - The camera fit reserves the panels at their scaled size
    (`hud_rects(viewport, seats, ui_scale)`), and the left column now
    follows the player panel down when its chips wrap
    (`position_left_column_below_hud`).
  - ⚠ **A wrapping chip row can wrap a chip that fits.** At 80 % the last
    chip dropped to a second line with room to spare — a rounding error
    against a container sized to its content — and hung below a panel sized
    for one. `fit_player_hud_width` turns wrapping on only when the chips'
    measured widths exceed the cap. The "(no mana)" placeholder is gone: an
    empty pool, the usual state, took a line of its own when the row
    wrapped.
- ✅ **The main menu is two sections** — Play (vs bot, draft, spectate,
  your own decklist) and Online (name, host, join) side by side under the
  format selector, with Settings and the developer tools (Audit Cards, Load
  Latest Debug State) in a small neutral row at the foot. It was one
  560-px column of nine buttons in seven unrelated colours that ran past
  the top of a 783-px window; it is ~645 px high at 100 %, green starts a
  game against the bot, blue opens another mode. The decklist buttons read
  "From File" / "From Clipboard" under "Your own decklist". The panel
  centres with auto margins in a scrollable root, so a window too short
  for it scrolls rather than cutting off the title. Layout harness:
  `--menu`, `--menu-format FORMAT`.

## Counters and card overlays (2026-09-27) — shipped

Read off layout-harness screenshots of a fixture board that now carries
counters (`layout_harness::COUNTERED`: +1/+1 stacks past the coin cap, two
kinds on one card, a −1/−1, a planeswalker, a saga, charge, stun, marked
damage, an opponent's poison), duel and pod, 1920x1080 and 1280x720.

- ✅ **Counters are coin piles down the card's left edge**
  (`counter_coins`): one pile per kind, a coin per counter up to five,
  the count on the top coin and the kind in a small tag beside it, on the
  side facing into the card (an opponent's cards face them; a tapped
  card's tag hangs below). The coins stacked *across* the card face, a
  diameter per counter, so an eleven-counter Walking Ballista drew a
  glowing tower the height of the card over its art and name, with a
  separate "+1/+1 ×11" label on top of the coins. Coins are the deep shade
  of their tag's colour and no longer bloom — a glowing coin washed out to
  pastel under its number.
  - A kind another overlay reads gets no coin (`board_counters`): a
    planeswalker's loyalty is its ◆ badge, a battle's defense its ◇
    badge, a creature's stun its "Stun N" status chip. Stun showed three
    times.
- ✅ **The P/T badge tones each stat and shows damage** (`pt_label`).
  Power and toughness are each green above the printed value and red
  below, so a +2/−1 says which half went which way (the whole badge took
  one tone, a tie broken by power). Marked damage comes off the toughness
  shown, in red, as Arena shows it: a 4/4 with 2 damage reads "4/2".
  Damage showed nowhere. The green/red ring drawn around a modified
  creature (`draw_pt_modified_overlays`) is gone — the badge says the same
  in numbers.
- ✅ **A card's overlays hide while another card lies over them**
  (`card::cover::CardCover`): counter piles (coins and label), P/T badges,
  keyword strips, token-pile, lock and regeneration badges. They are
  screen-space UI drawn over every card, so where cards overlap — a
  wrapped creature row, a tapped card turned under its neighbour — a
  covered card's overlays printed on the card on top: Serra Angel's "Fly
  Vig" read as Hangarback Walker's.
  - ⚠ **Test the card face under a pile, not the pile's top.** Stacked
    cards sit a few hundredths apart, so a pile on the card underneath
    rose through the card above; the covered-pile test passed and the
    coin showed through.
- ✅ **A changed count swells for a moment** (`theme::OverlayPulse`): a
  pile's number, a P/T badge — the change catches the eye instead of
  silently rewriting a digit.
- ✅ **The coin is a casino chip** (`coin_mesh`): turned from a profile
  with a rounded edge, a raised rim around a recessed cream face the count
  is printed on (in the kind's deep colour), a dark groove around the face
  and eight cream spots that wrap over the rim. It was a plain cylinder —
  a flat disc of paint with no edge to catch the light — backed by a
  second, larger cylinder as its outline. The colours are vertex colours,
  so the spots are geometry, not a decal fighting the surface for depth;
  every kind shares one material. A pile's chips are turned against each
  other and set a hair off-centre, and a chip a pile gains drops onto it
  (`CoinDrop`).
  - The count grows with its chip seen up close (Ctrl zoom), and the P/T
    badge sits over the printed P/T box at a size that follows the card:
    both were fixed-size and fixed-offset, so up close the count was a
    speck on a large face and the badge hung off the card's corner.
  - Layout harness: `--zoom-card NAME` holds the camera close over one of
    the viewer's cards.
- ✅ **Keyword strips sit on the card's P/T line** (`keyword_label`),
  ending at the P/T badge and running toward the card's left as seen. The
  bottom of a card is the part a card in front of it leaves showing, and
  on the card no neighbour's overlays land. Hung above the card's top edge
  (the display pass, above), an opponent's front row printed its strips
  over the P/T badges of the back row peeking out behind it — "Stun 1"
  under Tarmogoyf's 3/4. The strip ends where the badge's laid-out width
  says it starts (`ComputedNode`, a frame behind).
- ✅ **Damage numerals read, and say whose they are** (`impact`). A struck
  creature's "−N" punches in on its P/T box (the badge whose toughness
  just turned red) and rises off it; a batch's hits on one creature are
  summed. It was a thin red numeral at the card's centre, which vanished
  into card art, printed two hits over each other, and for a back-row
  creature landed on the card in front of it. Now outlined in near-black,
  brighter and larger. Layout harness: `--demo-damage` feeds the client a
  batch of damage a moment before the screenshot.
- ✅ **The viewer's own spell on the stack always has its 3-D card**
  (`sync_game_visuals`). It only came out of the viewer's hand, so a spell
  already on the stack when a view arrived — a reconnect, a resume, a
  spectator — or cast from elsewhere (a commander, a flashback) had no
  card on the table and no target arrows, which start at it. It drops
  onto the stack from above. Layout harness: `--stack`.

## Hits and deaths in light, the cards react (2026-09-28) — shipped

The table's last wireframe was its most dramatic moments: death bursts,
damage sparks, dig rings and mana motes were one-pixel gizmo rings and
spokes, as were the aura/equipment tethers and the active seat's outline.
Read off layout-harness screenshots (`--impacts AGE`, which fires a death,
a token death, three hits, a dig and — with `--stack` — mana, AGE seconds
before the shot) at 0.08-0.5 s, duel and pod.

- ✅ **Light as geometry** (`systems::glow`): soft shapes — orbs, pools,
  waves, trails, a frame — whose colour runs to nothing at their edges,
  blended additively (overlapping effects brighten rather than cover each
  other) and HDR so they bloom where bloom is on. Immediate-mode like
  `Arrows`: systems push `Light`s, `render_glow` builds one mesh a frame.
- ✅ **Deaths**: a flash on the felt, a shockwave ring out across the
  neighbouring cards, embers rising off the card and cooling
  (`impact::burst_lights`). The card itself burns first — its face flushes
  ember-red, chars to near black and shrinks for 0.45 s (`DeathBeat`)
  before it flies to the graveyard; it had flown there looking exactly like
  a bounced card. A token no longer blinks out: it shrinks away where it
  lay (`Vanishing`), after burning if it died. Both death events are read
  (a creature's may come as either), once per card.
- ✅ **Damage**: a flash, a small ring and streaking sparks that fly up
  and fall; a struck creature shudders side to side, further for a bigger
  hit (`animate::Jolt`).
- ✅ **Digs** (explore, discover): a cyan ripple and rising motes.
- ✅ **Mana**: each mote is a glowing orb with a trail, arcing from its land
  to the spell and bursting on arrival.
- ✅ **Cords and the active seat**: the aura/equipment tether is a cord in
  the arrows' style (`Arrows::cord`, no head, its light drifting into the
  host), brighter when either end is hovered; the active seat's board
  outline glows gold (`Light::Frame`). The client draws no gizmos now.
- Residual: all of it was seen only in stills; the motion (the jolt, the
  embers' drift, the beat's timing) is covered by unit tests, not by eye.

## Drag to act, and focus fades the overlays (2026-09-28) — shipped

- ✅ **Drag to act** (`systems::drag_act`). Drag a spell from the hand
  onto its target, an attacker onto the player (their HUD panel), the
  planeswalker or the battle it attacks, a blocker onto the attacker. A
  drag is the two clicks these always took in one gesture: the press is
  the first click, unchanged; letting go over something else is taken as
  a click there (`DragAct::release_click`, or the player's panel through
  `ButtonState::player_chip`) — by the input handler's own paths, so a
  drop is legal exactly when the click would be. A press and release in
  place is still just a click. Letting go over one of the viewer's own
  creatures while sending an attacker does nothing (a click there would
  add it to the attack), and dragging an attacker already in the plan
  keeps it there (its press took it out). While an attacker or blocker is
  dragged an arrow follows the pointer, snapping to what the release would
  pick (`gizmos::draw_drag_arrow`); the picked-up blocker's candidate
  arrows step aside for it.
  - Checked in a headless app (press, drag, release over a target) and
    end to end in the harness (`--combat drag`, a staged drag, whose
    release the input handler turned into the block). Not checked with a
    real mouse: the harness can't drive one.
- ✅ **Focus fades a dimmed card's overlays** (`focus::fade_card_overlays`):
  its P/T badge, keyword strip, counter labels, pile count and combat chip
  drop to 35 % opacity, and its 3-D counter chips take a darkened twin of
  their material. Left bright, they held the eye on the very cards that
  weren't choices. Overlays carry `CardOverlay(CardId)`; the fade scales
  from the opacities they had before it, so their own systems repainting
  them mid-fade doesn't compound it.

## Focus while choosing, a felt table, hover tilt (2026-09-28) — shipped

Read off layout-harness screenshots (`--combat target|blocks|plan`,
`--hover-card`, plain fixture; duel and pod, 1920x1080).

- ✅ **Focus while choosing** (`systems::focus`). While the viewer aims a
  spell or ability, picks attackers or picks blockers, every card on the
  table that isn't a choice dims to 38 % through its own face material,
  fading in and out. The choices: the target's legal set; the server's
  `legal_attackers` and the planeswalkers and battles an attack may aim
  at; its `legal_blockers` and the attackers. With no legal set to go by
  nothing dims. The hand doesn't dim. (A dimmed card's overlays stayed
  bright at first; they fade with it since — "Drag to act" above.)
- ✅ **A felt table** (`systems::table_cloth`). The ground and the seat tints
  share a generated, tiling felt texture (mottling, fibres, a faint weave,
  with its mip chain), and their vertex colours carry a pool of light
  fitted to the table's boards, falling off past them. Each seat's area
  has zones printed on it in its colour, as a playmat's: an outline round
  its board and a card slot under its deck and graveyard, which shows
  when the pile is empty.
- ✅ **A hovered battlefield card tilts toward the camera** as it lifts
  (`animate::HoverTilt`, 8° at full lift). A card's rotation is written
  outright by the layout, the tap animation and more, so the tilt is taken
  off in `First` and put back in `PostUpdate` before transform
  propagation: every other system sees and writes the card's own rotation,
  and the chips and borders parented to it tilt with it. No pivot entity.
- ✅ **Bug: a hovered card dropped when a view landed.** The layout's
  rebalance re-inserted every battlefield card's `CardHoverLift` with its
  lift zeroed, so an opponent acting under the pointer dropped the hovered
  card back onto the table while it stayed hovered. The rebalance now
  moves only the card's resting place.
- ✅ `CounterType::Ritual` (upstream) named in the client's counter labels;
  a pod's seat plate never wraps.

## Life that counts, compact token piles, a phase rail (2026-09-28) — shipped, with a residual

Read off layout-harness screenshots (`--life-change`, `--tokens`; duel and
pod, 1920x1080 and 1280x720).

- ✅ **Life totals count to their new value** (`game_ui::life_ticker`).
  Every readout — the viewer's badge, each opponent's row, a pod's seat
  plates — jumped to the new total. It now counts there over 0.75 s,
  lit red for a loss or green for a gain and swelling as it starts; a
  change landing mid-count carries on from what shows. The rows are
  rebuilt on each view, so the count starts before they are and they're
  built on its first frame.
- ✅ **A life numeral by the seat**, in the creature damage numerals' style
  (outlined, landing big; `impact::spawn_numeral`). The old one was a thin
  bare "-5" at fixed offsets from the screen corners that the HUD had
  since moved out from under: it printed over the hand and deck chips. In
  a pod it rises off the seat's name plate on the table (below the plate
  where the rise would run into a HUD panel); in a duel, beside the seat's
  HUD panel, level with its life. Beside the panels, a small window's
  top-left panel ran into the top-right one and the numerals overlapped.
- ✅ **Token piles stay compact** (`layout::stack_stagger`). Identical
  tokens already shared a pile with a ×N chip, but fanned like a land
  stack — each card a step out — so ten Goblins ran three and a half
  card-lengths over the neighbouring slots. A token pile fans three steps
  and stacks up from there. The chip is cream with dark print like the
  counter chips, on the top card's upper-right corner as seen, sized to
  the card, and swells when the count changes.
  - ⏳ Residual: a token with no art on disk (the harness's "Goblin",
    "Treasure") draws as a blank white card with its name in faint grey,
    and a pile of them reads poorly. See Token Labeling below.
- ✅ **The phase chart shows the turn's progress.** A rail down its left
  edge fills through the steps passed, in the colour of the seat whose turn
  it is (the viewer's own on their turn, an opponent's on theirs), with the
  current row lit in it; steps passed recede.
- ✅ **Layout harness:** `--tokens` adds piles to two seats;
  `--life-change` swings every seat's life a moment before the
  screenshot.

## Combat and targeting arrows as geometry (2026-09-28) — shipped

Read off layout-harness screenshots of every combat and targeting state
(`--combat blocks|declared|plan|target`, duel and pod, 1920x1080 and
1280x720).

- ✅ **Arrows are shaded ribbons, not gizmo lines** (`systems::arrows`).
  Attack, block, stack and target-drag arrows were 3-5 px lines that read
  as debug output next to the lit cards and chips. An arrow now arcs over
  the table from source to target, turned to face the camera; it has a
  dark rim (it reads over bright art), a flat body in its cue's colour, a
  thin glowing core with bands of light running toward the target, a
  tail that fades out of its source and a broad head. It grows out of its
  source when it appears and fades when it goes. Immediate-mode like
  gizmos: systems push to `Arrows`, `render_arrows` keeps one mesh per
  `ArrowKey` across frames. The legal-target, decision-source and
  defender rings are the same geometry, lying on the card's face.
  - Tuning notes: past 1.0 the tonemapper and bloom turned a thin yellow
    arrow to cream, so the body stays in range and only a thin core line
    glows.
- ✅ **Combat chips replace the swords and diamonds** (`combat_badge`).
  ⚔ on an attacker, 🛡 on a blocker, coloured with its part in the combat:
  orange attacking, cyan a declared block, green the viewer's planned
  block (and the attacker it blocks), gold the blocker picked up, red an
  attacker still unblocked. A chip sits at the top edge as seen — on an
  opponent's upside-down card, toward the printed left, clear of the P/T
  badge and keyword strip — or, when another card lies over that edge, at
  the top of what shows.
- ✅ **Arrows meet what shows.** A card covered by its neighbour is met in
  the middle of its visible part (`CardCover::visible_centre`); an arrow
  aimed at its centre landed on the card lying over it. Block and attack
  arrows run chip to chip and stop at the chips' rims, the stop measured
  as seen (an arrow coming in toward the camera leaves more of its curve
  bare).
- ✅ **Bug: a spell's legal-target rings never showed.**
  `spawn_decision_ui` cleared `LegalTargets` on every frame without a
  decision for the viewer — every frame of a cast's own targeting
  session — so the rings `enumerate_for_cast` filled in were gone a frame
  later, and a click on anything clickable was taken as the target. The
  set now stays while a cast's session is open
  (`casting_keeps_legal_targets`). A click off the legal set during a cast
  is now ignored, as the click handler always meant.
- ✅ **Layout harness:** `--combat SCENE` stages a combat or a targeting
  pick client-side (patched view, set plans, auto-pass held).

## The stack beside the table, and a board that answers the cursor (2026-09-28) — shipped

- ✅ **The 3-D stack hangs in a lane beside the table**
  (`framing::stack_lane`, `CameraHome::stack_lane`). It lay flat across
  the table's centre, where a spell covered the creatures it was aimed at.
  Each camera fit searches the right half of the window, as the fit
  itself does, for the pile of three cards that covers the least of the
  representative board, clear of the HUD — then the larger card, the spot
  further right. The cards face the camera, well in front of the table so
  nothing on it draws through them; the oldest item is at the top and
  each newer one a step lower and on top, so what resolves next is wholly
  in view. A duel's table leaves the right side empty and the lane covers
  nothing (`framing::tests::the_stack_lane_hangs_clear_of_the_board`); a
  pod's table fills the window and the lane takes its least-used edge.
  Target arrows run from the lane.
  - The 2-D stack panel sits beside the lane, top-aligned on its left
    (`place_stack_panel`), without its card thumbnails — the lane shows the
    cards, and a row's hover still opens one. Bottom-centre it lay over
    the viewer's lands, and at 1280x720 over their creatures.
  - `PlayCardAnimation` lands at a `target_scale` (1 on the table, the
    lane's on the stack); it always ended at 1, so a lane card flew in at
    table size and ran off the window's edge.
- ✅ **A hovered battlefield card lifts and grows** (`BF_HOVER_LIFT`,
  `BF_HOVER_GROW`): it rises off the table — its shadow slides out — and
  scales up 6 %, and what sits on it (chips, badges) follows. Only a hand
  card lifted; a battlefield card sat still while its preview appeared.
  - The tilt toward the camera followed (2026-09-28, "Focus while
    choosing, a felt table, hover tilt" above) without a pivot.
- ✅ **Layout harness:** a `--screenshot` run ignores the mouse (the
  desktop cursor over the window hovered a card and popped its preview
  into the shot); `--hover-card NAME` hovers one on purpose;
  `--mana-gallery` also lays out a real option-ballot modal.

## Mana symbols (2026-09-27) — shipped, with residuals

- ✅ **Costs draw as mana pips** (`mana_text`). The engine writes every
  cost the Oracle way (`ManaCost::summary()`'s `{3}{W}{W}`, an ability's
  `{2}{T}: Draw a card`) and the client printed the braces. `ManaText` on
  a node draws such a string as a row: each `{…}` a round pip in the
  printed symbol's colour with its letter or number — hybrids split corner
  to corner, `{T}` ↻, `{Q}` ⟲, `{E}` ⚡ — and the rest as text, wrapping
  between words where asked. The UI font has no mana glyphs, so a pip is a
  node. On it: the command-zone cost chip, the ability / alternative-cost
  / split-card / spree popups, the hand menu, the Alt tooltip, the hover
  panel's cost lines, the "{1}{U} to go" manual-tap banner, optional
  trigger prompts ("Pay {3} to keep this trigger?") and the devotion chip
  ("◆ {W}6 {U}2"). The mana-pool chips took the same palette, so a pool's
  white is a cost's white. Layout harness: `--mana-gallery`.
  - ✅ Phyrexian pips carry Φ. The fallback subsets hold only the symbols
    the client uses, so Noto Sans Math's was regenerated
    (`scripts/ui_fallback_fonts.py --only NotoSansMath`: `--only` rewrites
    one subset and keeps the rest, so it needs no Noto Emoji source, which
    `noto-fonts` doesn't ship; `--list` had crashed without `--emoji`).
    ⚠ **A glyph in a fallback font is reached only from a run in a script
    the fallbacks are registered under** (`theme::FALLBACK_SCRIPTS`). Φ is
    a Greek letter, not a symbol: with its glyph in the subset it still drew
    as a box until Greek was added. `every_ui_symbol_has_a_glyph` now checks
    each symbol's script too (`icu_properties`, a dev-dependency already in
    the tree through parley).
  - ✅ The decision modals' titles and option buttons draw costs as
    pips too (`ChooseOption` ballots — "Pay {2}" for ward —, the X
    picker's prompt, both mode pickers), through `mana_text::spawn_text`:
    plain text as before, wrapping mana text when there is a symbol.
    ⏳ A target prompt's description reaches the hint chip as text.

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
  - ⏳ Residual: the log renders newest-first, so a new entry arriving
    while the player is scrolled back shifts the view by a row. (It no
    longer rebuilds every row on each event — 2026-09-29, `LogRow` — but a
    row inserted at the top still moves what's below it.)
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
- ✅ **`UiScale` was a stub, so text scaling was impossible** — shipped
  2026-09-27 as the UI size setting ("UI size and the menu", above).

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

### Exile Zone Browser
✅ Shipped — `V` toggles a browser listing exiled cards with per-card
source annotations (linked exile, cipher, foretell, …).

### Card Tooltip with Full Oracle Text
✅ Shipped 2026-09-29 — the hover preview and the Alt peek print the card's
Oracle text (`card::oracle`, compiled in from `assets/oracle.tsv`): type
line and stats, every paragraph, both halves of a split / adventure /
prepare card, the shown face of a double-faced one, and a reminder for each
keyword the text doesn't explain. Tokens the catalog can't name read from
the view (type line, keywords, the engine's ability labels). Residual ⏳:
the table covers 22,144 of the catalog's 25,884 names (the rest are
synthesized cards Scryfall doesn't know) and goes stale as the catalog
grows — a newer card falls back to the phrased lines until
`CRAB_BLESS_ORACLE=1 cargo test -p crabomination_client oracle` rebuilds
it from `scripts/.scryfall_cache.json`.

### Graveyard Order and Timestamps
✅ Order shipped 2026-09-29 — the graveyard browser lists newest first (the
engine's last card is the top) and says so in its header. Residual ⏳: no
per-card "went there on turn N" timestamps; the view doesn't carry them.

### Token Labeling
✅ Shipped 2026-09-29 — art-less tokens get a drawn face (`card::proxy`),
and token art is fetched by exact name. Residual ⏳: downloaded token art is
the first printing *named* like the token, which can still differ from the
game's token in colour or P/T (the P/T badge corrects the numbers).

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
  `token_badge.rs`); same-name lands already stacked. ✅ Pods seat two to
  an edge (`layout::pod_frame`; three players make a triangle), each seat
  laid out like a 1v1 near edge with a pile strip either side: five groups a
  row on every board, up from three and a half on each far board when all
  three opponents shared the far edge, and a wrapped creature row gets a
  whole card of depth instead of a shingle. ✅ Each aura/equipment is
  joined to its host by a cord (`gizmos::draw_attachment_tethers`).
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
  resolution presets, maximize-on-launch, render quality, UI size (also in
  the in-game Esc menu), animation speed, hand sorting — applied live and
  persisted. Remaining ⏳:
  keybind remapping, audio (once there is audio).
- 🟡 **Deck library** — Commander's deck picker (2026-09-26,
  `deck_picker.rs`) picks your stock deck and each bot's from the 183 stock
  lists, with search. Paste-from-clipboard import ✅ (menu decklist "From
  Clipboard", lobby "Paste Deck"). Power tiers ✅ (2026-09-27, `pod::power`):
  each row shows the deck's tier 1-5 from its measured six-seat win share
  under bot play (`scripts/pod_power.sh` regenerates the table), and a bot
  seat left on Random is dealt a deck within one tier of yours — the same
  decks ran 0-67 %, so a random pod could seat a two-in-three winner against
  decks that almost never win. Your default deck is a random middle-tier one
  each launch (it was Sigarda, tier 1, every time). ✅ Imported decks are
  saved and listed ("Your Decks", 2026-09-30). Remaining ⏳: the lobby's
  bot seats still take stock decks unbalanced.
- ✅ **Deck import says what's wrong, all of it** (2026-09-26,
  `deck_import.rs`) — a list that can't play opens a report of every
  problem (each unknown card with the names it probably meant, or each
  format rule it breaks) with Copy list / Close; the status line had named
  four unknown cards and stopped. Names now match past accents and curly
  quotes. Layout harness: `--import-report PATH`.
- ✅ **Symbols drew as boxes** (2026-09-26) — the UI font (Mirano Extended
  Light) has 13 of the ~80 non-ASCII symbols the client prints, and Bevy's
  shaper (parley) falls back only to fonts it is handed, so ☠ 👑 ♥ ✋ ⚔ ▶ ▼
  ─ and 60 more were empty boxes on every screen. Four Noto subsets
  (~36 KB, OFL) are registered as fontique fallbacks
  (`theme::register_symbol_fallbacks`); `scripts/ui_fallback_fonts.py`
  rebuilds them and `theme::tests::every_ui_symbol_has_a_glyph` fails on a
  new symbol without a glyph.
- ✅ **Knocked out of a pod** (2026-09-26) — the game-over screen ranks the
  table (`GameState::placement`: "You finished 3rd of 4", each seat's turn
  and cause); the log says who went out (`GameEvent::PlayerLost`); your
  own HUD row collapses to "☠ OUT · cause" and the turn buttons hide; a
  "You're out" panel (Keep watching / Leave game) shows if the game is
  still going 2.5 s later — a local pod's bots finish in about a second,
  so it is for a network pod. Layout harness: `--viewer-out`,
  `--hold-seat N`, `--deck-picker`.
- ✅ **A player drops from a network game** (2026-09-26) — the log says so
  ("Ana disconnected — 60 s to reconnect before they concede", then
  "reconnected" or "didn't reconnect" + the concession) and the table plays
  on; before, every other seat waited on the empty chair for good.
- ⏳ **Bo3 + sideboarding UI** — Learn/Lessons sideboard plumbing exists
  engine-side.
- ⏳ **Replay viewer** — see "Replay scrubber" (Tier 3 below).
- ⏳ **Accessibility pass** — colorblind-safe target rings (shape, not
  only color), reduced-motion toggle, finish keyboard-only play; see
  "Theme variants". (Text scaling ✅ — the UI size setting.)

### Conspire cast UI (follow-up)
✅ Shipped 2026-09-30 — right-click opens the two-creature picker and
submits `CastSpellConspire` (see the section at the top).

### UI Roadmap (push claude/modern_decks — session-derived)

Ordering layer over the detailed items below. Cross-references existing
entries instead of duplicating; tiers ordered by start-here leverage.

**Player Crest track** — promote 3-D disc into stat readout + state
indicator + click target. Slims the 2-D chip strip.
- Phase 3 ✅ — damage/heal floaters: counting readouts and a numeral by
  each seat (`game_ui::life_ticker`, 2026-09-28).
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
- Token stacking ✅ — identical tokens pile with a ×N chip; piles fan
  only three steps (2026-09-28).
- Valid-target affordance ✅ — non-choices dim while choosing
  (`systems::focus`, 2026-09-28); legal targets carry pulsing rings.
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

### Selective Attacker Picking
✅ Click-based per-attacker picking is wired (`game_ui/mod.rs`, the
"Attacker selection" block): click an own creature to toggle it into the
plan, click an opponent planeswalker / player disc / 2-D HUD chip to
reassign the last-added attacker's defender, Esc / right-click to clear,
and `A` / the Attack button submits the picked plan (falling back to
"attack all eligible at next opp" when the plan is empty). Selected
attackers show their ⚔ chip (`combat_badge.rs`) and an arrow to their
defender.

✅ Drag an attacker onto its defender (player panel, planeswalker, battle)
as an alternative to click-to-assign (`systems::drag_act`, 2026-09-28).

### Hover-Dwell Card Preview
✅ Shipped as `ui::hover_card_preview`: hovering a card shows its art and
Oracle text beside it, clamped to the viewport. Since 2026-09-29 it sits
beside the card's projected rect rather than the cursor, so it never
covers the card it previews (the stack/log previews in `ui_card_hover`
sit beside their row the same way). ✅ Since 2026-09-30 it opens after a
300 ms dwell and fades in (see the section at the top).

### Decision Modal vs 3-D Hand Consistency
Mulligan and PutOnLibrary modals are transparent overlays over the 3-D
hand (player clicks the 3-D cards). Scry / Search / Discard render their
own 2-D card grid. No design rule says which decisions go which way, so
users can't predict whether to click the 2-D modal cards or the 3-D table
cards. Pick one rule (e.g., "decisions on the viewer's own hand → 3-D +
banner; decisions on hidden zones → 2-D modal grid") and migrate.

### Right-Click Action Hint
✅ Shipped 2026-09-30 — a 🖰 chip names what right-click does with the
card (`hand_chips`; see the section at the top).

### Hand-Fan Spacing for Large Hands
✅ Shipped: past `HAND_FAN_SOFT_CAP` (7) the spacing shrinks to keep the
seven-card width, and since 2026-09-30 the droop and tilt follow the
card's place along that width (`layout::fan_arc`), so a 15-card hand keeps
the seven-card arc instead of sinking its ends below the window.

### Drag-and-Drop for Hand → Battlefield
✅ Drag-to-target shipped (`systems::drag_act`, 2026-09-28): drag a spell
onto its target. Hand cards still play on the press (a creature or land
casts the moment it's pressed), so there's no drag-to-position; the card
itself doesn't follow the pointer, the aiming arrow does.

### Settings Menu
The animation-speed slider is currently wedged into the quality panel
(`quality.rs::setup_quality_panel`). A proper Settings panel (audio,
key rebinds, accessibility) would cleanly separate these and give a
natural home for future global preferences. (UI size ✅, in both.)

### Auto-Pass Toggle
`auto_advance_p0` (`game_ui.rs:2000+`) decides for the player when to pass
priority. A toolbar toggle ("Auto-pass: On/Off") lets new players step
through their own turn priority-by-priority instead of having the engine
fast-forward.

### Alt-Peek Inside Decision Modals
✅ Shipped 2026-09-30: every card tile in a decision modal previews with
its rules text on hover and opens the large peek under Alt (see the
section at the top).

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
(`UiScale` ✅ 2026-09-27: Auto is 200 % at 2160 px high, so a 4K window
lays out as 1080p doubled rather than as a small island.)

🟡 **The table half is done (2026-09-23); the HUD half is not.** The camera
is now fitted to the window (`card/framing.rs`): the closest pose that keeps
a busy board on screen and clear of the corner panels, refit on resize. With
the opponent's 1v1 hand moved beside their deck and the viewer's hand off
their land row, 1v1 cards at 1920x1080 went 121 → 129 px (242 → 279 at 4K);
pods, now seated two to an edge, went 80 → 99 px (160 → 212 at 4K) with
every board wider. `framing::tests::budget` gates the numbers. The panels
themselves are still fixed-px, and **they are what bounds the table**. The
action buttons moved from the near-left corner (the widest part of the
view) up under the phase chart, and Export State / Surrender / the
duplicate Leave went to the Esc menu: pods 86/77 → 99/84 px, 1v1 125/114 →
129/118. Next to bind: the game log for a pod (96/89 px without it), the
phase chart for a 1v1 (136/124). The player panel's chips now wrap at the
width the fit reserves (2026-09-27, above), so it no longer runs under the
opponent panel at 1280x720. The fit only knows the panels by the nominal rects
in `framing::hud_rects` — a slot system that reported real rects would let
it use space an empty log or a short chip row leaves.

---
