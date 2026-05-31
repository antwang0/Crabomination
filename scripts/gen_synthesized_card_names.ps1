<#
.SYNOPSIS
  Regenerate crabomination_client/src/synthesized_cards.rs.

.DESCRIPTION
  The STX catalog contains ~3700 "synthesised" cards -- college-flavoured
  variety invented to flesh out the audit/cube pool. None have a real
  Magic printing, so Scryfall 404s every one of them. Rather than detect
  them with fragile name heuristics, we determine the exact set offline:

      synthesised = { STX catalog card names } - { real Scryfall names }

  The authoritative real-name list comes from Scryfall's `catalog/card-names`
  endpoint (every card name Scryfall knows, ~34k). Names that are also
  defined by a non-STX catalog factory (real reprints / tokens) are excluded
  so the manifest stays pure "STX-invented cards".

  Run this whenever STX card definitions are added or renamed. A stale entry
  is harmless (a real card just downloads as normal); a missing entry only
  costs a single one-time 404 on the next prefetch.

.NOTES
  Requires network access (one request to api.scryfall.com).
#>

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$stxDir = Join-Path $repo 'crabomination_catalog\src\sets\stx'
$out = Join-Path $repo 'crabomination_client\src\synthesized_cards.rs'

Write-Host 'Fetching Scryfall card-name catalog...'
$headers = @{ 'User-Agent' = 'Crabomination-audit/1.0'; 'Accept' = 'application/json' }
$resp = Invoke-RestMethod -Uri 'https://api.scryfall.com/catalog/card-names' -Headers $headers -TimeoutSec 60
$real = @{}
foreach ($n in $resp.data) { $real[$n.ToLower()] = $true }
Write-Host "  $($resp.data.Count) real card names."

# Names defined outside the STX module (real reprints / tokens) -> exclude.
$exclude = @{}
$nonStx = Get-ChildItem -Recurse (Join-Path $repo 'crabomination_catalog\src\sets\*.rs') |
    Where-Object { $_.FullName -notmatch '\\stx\\' }
foreach ($f in $nonStx) {
    foreach ($m in [regex]::Matches((Get-Content $f.FullName -Raw), 'name: "([^"]*)"')) {
        $exclude[$m.Groups[1].Value.ToLower()] = $true
    }
}

# Every card name defined in the STX module. We take the *first* `name:`
# field after each `pub fn ...() -> CardDefinition` (the card's own name),
# not every `name:` in the file -- the latter would also pick up nested
# token names inside a card's effect.
$stxNames = @{}
foreach ($f in (Get-ChildItem (Join-Path $stxDir '*.rs') |
        Where-Object { $_.Name -notin @('all_factories.rs', 'mod.rs') })) {
    $curFn = $false
    foreach ($ln in (Get-Content $f.FullName)) {
        if ($ln -match '^\s*pub fn [a-z0-9_]+\(\) -> CardDefinition') { $curFn = $true; continue }
        if ($curFn) {
            $m = [regex]::Match($ln, '^\s*name: "([^"]*)"')
            if ($m.Success) { $stxNames[$m.Groups[1].Value] = $true; $curFn = $false }
        }
    }
}

# synthesised = STX names not on Scryfall and not a real entry elsewhere.
$synth = $stxNames.Keys |
    Where-Object { -not $real.ContainsKey($_.ToLower()) -and -not $exclude.ContainsKey($_.ToLower()) } |
    Sort-Object -Unique
$tmp = [string[]]$synth
[Array]::Sort($tmp, [System.StringComparer]::Ordinal)
$today = (Get-Date).ToString('yyyy-MM-dd')

$L = New-Object System.Collections.Generic.List[string]
$L.Add('//! Synthesized (catalog-invented) STX card names -- GENERATED, do not edit by hand.')
$L.Add('//!')
$L.Add('//! These cards have no real Magic printing, so Scryfall 404s every one of')
$L.Add('//! them. The list is the exact set difference of the STX catalog''s `name:`')
$L.Add('//! fields minus Scryfall''s `catalog/card-names` endpoint (and minus names')
$L.Add('//! that are real catalog entries / tokens defined outside the STX module),')
$L.Add('//! computed offline. Regenerate with:')
$L.Add('//!')
$L.Add('//!     powershell -File scripts/gen_synthesized_card_names.ps1')
$L.Add('//!')
$L.Add('//! whenever STX card definitions are added or renamed. A stale entry is')
$L.Add('//! harmless (a real card just downloads as normal); a missing entry only')
$L.Add('//! costs a single one-time 404 on the next prefetch.')
$L.Add('//!')
$L.Add("//! $($tmp.Count) names as of $today.")
$L.Add('')
$L.Add('use std::collections::HashSet;')
$L.Add('use std::sync::LazyLock;')
$L.Add('')
$L.Add('/// Case-insensitive membership test: is `name` a catalog-synthesized STX')
$L.Add('/// card with no real Scryfall printing (and therefore safe to stamp with a')
$L.Add('/// cardback placeholder without attempting a download)?')
$L.Add('pub fn is_synthesized_card(name: &str) -> bool {')
$L.Add('    static SET: LazyLock<HashSet<String>> = LazyLock::new(|| {')
$L.Add('        SYNTHESIZED_STX_CARDS')
$L.Add('            .iter()')
$L.Add('            .map(|s| s.to_ascii_lowercase())')
$L.Add('            .collect()')
$L.Add('    });')
$L.Add('    SET.contains(&name.to_ascii_lowercase())')
$L.Add('}')
$L.Add('')
$L.Add('#[rustfmt::skip]')
$L.Add('const SYNTHESIZED_STX_CARDS: &[&str] = &[')
foreach ($n in $tmp) { $L.Add('    "' + $n + '",') }
$L.Add('];')

($L -join "`n") + "`n" | Set-Content -Path $out -Encoding utf8 -NoNewline
Write-Host "Wrote $out with $($tmp.Count) synthesized names."
