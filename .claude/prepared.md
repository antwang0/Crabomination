# Prepare Mechanic — quick reference

Two halves; only Half 1 ships today.

## Half 1 — Prepared cards (the spell side, **wired**)

A "prepared card" is a creature with a back-face *prepare spell* — a
fully castable spell on the back of an otherwise-vanilla creature
front. Mechanically these ride the engine's existing MDFC plumbing
(`back_face: Some(Box<CardDefinition>)` + the
`GameAction::CastSpell` / `CastSpellBack` action pair). The
distinguishing feature vs. a real Scryfall MDFC (e.g. pathways) is
that the **front+back pair is engine-invented** — the front creature
and back spell each exist in real Magic, but Scryfall has no record of
them glued together as a double-faced printing.

**Catalog:** `crabomination/src/catalog/sets/sos/mdfcs.rs` (plus a few
in `sos/creatures.rs`). Helpers: `vanilla_front(...)`, `spell_back(...)`.

**Examples:** Spellbook Seeker // Careful Study, Pigment Wrangler //
Striking Palette, Cheerful Osteomancer // Raise Dead, Joined
Researchers // Secret Rendezvous, Adventurous Eater // Have a Bite,
Quill-Blade Laureate // Twofold Intent, Spiritcall Enthusiast //
Scrollboost, Landscape Painter // Vibrant Idea, Scathing Shadelock //
Venomous Words, Kirol, History Buff // Pack a Punch.

**Image prefetch:** `crabomination_client::scryfall::download_card_image`
queries the front name with `face=back` first (the real-MDFC path).
On HTTP 422 (Scryfall: front isn't double-faced) **or** 404 (front not
on Scryfall), it falls back to a direct `cards/named` lookup of the
back name. The back name is always a real Scryfall card on its own,
so the fallback always succeeds and the runtime renders the spell's
real art rather than a cardback placeholder. Saved under
`<back>_back.png` to coexist with any unrelated `<back>.png` front.

If you add a new prepared card, the prefetch handles it automatically
— no hand-maintained allowlist.

## Half 2 — The prepared flag (**⏳ pending**)

A per-permanent boolean toggled by `becomes prepared` /
`becomes unprepared` effects. Distinct from Half 1: the flag mechanic
*reads* whether a creature has a prepare spell (its `back_face`) but
the flag itself is independent state on the permanent.

**Toggle cards** (currently ⏳ — no flag primitive yet):
- Biblioplex Tomekeeper — `{4}` 3/4 with ETB toggle (prepare or
  unprepare a target).
- Skycoach Waypoint — colorless land with `{3},{T}: prepare target`.

**Payoff cards** carry a `Prepare {cost}` activated/triggered ability
gated on the flag, with reminder text "(Only creatures with prepare
spells can become prepared.)" — i.e. flag-toggle effects must reject
targets whose `back_face` is `None`.

**Engine work needed:**
1. `PermanentFlag::Prepared` (or `CounterType::Prepared` count-1) on
   `Permanent`, surfaced through `PermanentView` for the client UI.
2. `Effect::SetPrepared { what, value: bool }`.
3. `Predicate::IsPrepared` for prepare-payoff conditional clauses.
4. Oracle-text helper that wires "Prepare {cost}: …" into a standard
   activated ability with `gate: IsPrepared`.

See STRIXHAVEN2.md → "Prepare mechanic" and TODO.md → "Prepare
Mechanic (SOS)" for the full status.
