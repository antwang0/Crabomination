---
name: verifier-client
description: >-
  Evidence-capture protocol for visually verifying crabomination_client (the
  Bevy 0.18 GUI). Launches the release build, drives it with synthetic input,
  and screen-captures the window so HUD/3-D changes can actually be observed.
  Use when the verify/run flow needs to LOOK at the game client. LOCAL ONLY —
  requires a Windows desktop with a real GPU + interactive display; does NOT
  work in headless/remote (cloud routine) environments.
---

# Visually verifying crabomination_client

The surface is pixels. You launch the app, drive it to where the changed UI
renders, and screen-capture the window. Captured PNGs are the evidence.

## Environment requirement (read first)

This needs a **local Windows session with a real GPU and a visible desktop**.
The client renders through wgpu/Vulkan and opens a winit window. A headless or
remote agent (a `/schedule` cloud routine) has no display/GPU and **cannot run
this** — it could only do the compile/test half, which is exactly the
substitute `verify` warns against. Recurring visual checks must run locally
(`/loop` in an interactive session, or Task Scheduler), not as a cloud routine.

## Build

```powershell
cargo build --release -p crabomination_client
```

- **Release, not debug.** Debug links too slowly, and the standalone exe is what
  you screen-capture.
- The release profile must keep Bevy's `dynamic_linking` feature **off** (it is,
  by default — it's opt-in behind `--features dynamic_linking` / `cargo dev`).
  With it on, the LTO release link fails under rust-lld.

## Make the exe find assets (critical — no fonts ⇒ no text)

Run directly, the exe resolves `assets` relative to **its own dir**
(`target/release/assets`), not the crate. Junction it (no admin needed):

```powershell
cmd /c mklink /J "C:\Users\JB\repos\Crabomination\target\release\assets" `
                 "C:\Users\JB\repos\Crabomination\crabomination_client\assets"
```

That exposes `fonts/` (without it, all text is invisible), `cardback.png`,
`models/`, and the cached `cards/` art (~5k images locally). Remove it on
cleanup: `cmd /c rmdir "...\target\release\assets"`.

## Launch into a game (skip the menu)

```powershell
$exe  = "C:\Users\JB\repos\Crabomination\target\release\crabomination_client.exe"
$work = "C:\Users\JB\repos\Crabomination\crabomination_client"
$snap = "$work\debug\deadlock-t24-1777414367-502430300.json"
$p = Start-Process $exe -ArgumentList @("--load-state",$snap) -WorkingDirectory $work -PassThru
Start-Sleep -Seconds 9   # window create + asset load + first-view render
```

`--load-state` boots straight to `AppState::InGame`. **Caveat:** the
`debug/deadlock-*.json` snapshots are stale (`unknown card name: "Bird"`) and
**fall back to a fresh local bot match** — fine for HUD verification, but you
can't stage a specific board this way. (Fixing/regenerating a snapshot would let
you land on an exact state.)

## Capture (screen-region, NOT PrintWindow)

PrintWindow / BitBlt-from-window return **black** for the GPU surface. Copy the
screen region under the window instead, and force the window topmost at a known
rect first (otherwise you capture whatever's in front — e.g. the IDE).

```powershell
Add-Type @"
using System; using System.Runtime.InteropServices;
public class K {
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h,IntPtr a,int x,int y,int cx,int cy,uint f);
  [DllImport("user32.dll")] public static extern IntPtr PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
  public static void Key(IntPtr h,int vk){ PostMessage(h,0x100,(IntPtr)vk,IntPtr.Zero); System.Threading.Thread.Sleep(40); PostMessage(h,0x101,(IntPtr)vk,IntPtr.Zero); }
}
"@
Add-Type -AssemblyName System.Drawing
# The client now opens MAXIMIZED (see `maximize_window` in main.rs), so it
# already fills the display. Bring it topmost WITHOUT moving/resizing
# (SWP_NOMOVE|NOSIZE|SHOWWINDOW = 0x43) and capture the monitor work area.
$W=2560; $Ht=1392   # your primary work-area size (Screen.WorkingArea)
$h=(Get-Process crabomination_client|Select-Object -First 1).MainWindowHandle
function Top { [K]::SetWindowPos($h,[IntPtr](-1),0,0,0,0,0x43)|Out-Null; Start-Sleep -Milliseconds 300 }
function Shot($o){ $b=New-Object System.Drawing.Bitmap $W,$Ht; $g=[System.Drawing.Graphics]::FromImage($b); $g.CopyFromScreen(0,0,0,0,(New-Object System.Drawing.Size $W,$Ht)); $b.Save($o,'Png'); $g.Dispose();$b.Dispose() }
function Crop($src,$x,$y,$w,$hh,$o){ $s=[System.Drawing.Image]::FromFile($src); $c=New-Object System.Drawing.Bitmap $w,$hh; $cg=[System.Drawing.Graphics]::FromImage($c); $cg.DrawImage($s,(New-Object System.Drawing.Rectangle 0,0,$w,$hh),(New-Object System.Drawing.Rectangle $x,$y,$w,$hh),'Pixel'); $c.Save($o,'Png'); $cg.Dispose();$c.Dispose();$s.Dispose() }
```

`Read` the saved PNG to view it (it downscales the full frame; `Crop` regions for
fine detail like card borders or log text).

## Drive with PostMessage (NOT SendKeys)

`SendKeys`/`AppActivate` is intermittent against the winit window. `PostMessage`
of `WM_KEYDOWN/UP` straight to the handle is reliable.

```powershell
Top
[K]::Key($h,0x4B); Start-Sleep -Seconds 2     # K  = keep opening hand (clears the mulligan modal)
foreach($i in 1..8){ [K]::Key($h,0x4E); Start-Sleep -Seconds 2 }   # N = Next Turn, cycles the bot's turn
Top; Shot "$env:TEMP\shot.png"
[K]::Key($h,0x70)                              # F1 = shortcut overlay
```

Virtual-keys: `K`=0x4B keep · `M`=0x4D mulligan · `N`=0x4E next-turn · `Space`=0x20
pass · `E`=0x45 end-turn · `A`=0x41 attack-all · `F1`=0x70 help. **You must press
`K` first** — the mulligan modal blocks turn input, and `N` is ignored until it's
dismissed.

## Cleanup

```powershell
Stop-Process -Id $p.Id -Force
cmd /c rmdir "C:\Users\JB\repos\Crabomination\target\release\assets"   # remove the junction
```

## What you can / can't observe this way

**Reliable:** app health (runs, no crash), font/glyph rendering (tofu check),
HUD layout — turn line, phase chart (`▶` active-step), player chips, decision
modals, the **F1 shortcut overlay**, battlefield permanents.

**Gotchas that blocked observation:**
- **The viewer hand fan never renders in the `--load-state` local-bot
  fallback** — confirmed absent at every turn AND at a full 2560×1392 window
  (so it's not a window-height clip). The fallback game most likely never
  fires the draw events that spawn the hand-card visuals. Consequence: the
  green *castable* and red *will-die* borders (which only attach to
  hand/battlefield cards) **can't be verified this way** — you need a real
  networked match with a normally-dealt hand. (If you regenerate a valid
  `--load-state` snapshot so it loads instead of falling back, the hand may
  appear; untested.)
- **Game log renders empty** in the `--load-state` local-bot fallback — the
  log feeds off `LatestServerEvents`, which that path may not populate. Log
  features (dividers, `×N` coalescing, ▼/✖/▲ glyphs) likely need a real
  networked match to observe.
- **Transient effects** (phase banner ~1.5s, life-change flash ~1.3s) need a
  capture timed to the moment — loop `Shot` right after a transition.
- **Bot may never attack**, so combat states (dying-creature borders, life
  flashes from damage) may not arise just by passing turns; you may have to
  actually declare attackers (`A`) with your own creatures in play.
