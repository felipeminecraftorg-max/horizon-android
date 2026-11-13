# Horizon XMB Mobile — five deliveries

**Owner-directed rebuild of the Android launcher Old School preset**. The sole
historical reference for interaction and atmosphere is PlayStation 3 XMB; the
result must be original Horizon art and code, optimized for portrait and
landscape Android touch. The PS3's horizontal category axis crosses a vertical
item axis, with one dominant focus and the content moving around it.

## Invariants for every delivery

- Retire the former Old School dashboard, tiles, cards, old icon set, old font
  styling, chrome backdrop and visual composition. **Do not call** its former
  Home compositor, including a temporary compatibility fallback. The existing
  app-state and engine functions are reused only as back-end services.
- Keep Liquid Glass and other presets completely independent and unchanged.
- Preserve the packaged Old School user SFX library, including the existing
  audio decoding and persisted sound preferences. Do not copy Sony recordings.
- The user explicitly requested five substantial development deliveries, each
  performed after their next "continuar" message, not dozens of minor patches.
- New graphics, motion, font licenses and icons must be original or appropriately
  licensed. Do not ship PS3 proprietary themes, icons, sound or firmware data.

## Part 1 — structural, running XMB core

- Dedicated Rust \`ui_theme::launcher_xmb\` module, independent compositor.
- New horizontal category axis, selected icon and contextual category label.
- Virtualized vertical item column with a stable selected row.
- Portrait and landscape cross geometry and touch target sizes.
- Persistent Home selection; D-pad, keyboard and basic axis-aligned touch swipe.
- Home items route to the existing 8 working Horizon actions; surrounding catalog
  pages are still the existing pages until Part 3.
- Automated native UI geometry/focus tests; no old dashboard tiles rendered.
- **Limit:** the simple background and schematic icons are interim foundations
  for Part 2, and dense secondary pages remain due for full replacement.

## Part 2 — visual and material identity

- Native WGSL background: layered procedural ribbons, moving aurora energy,
  slow phase, soft color transitions, subtle focus radiance and smooth mobile
  GPU blending. Must not use the previous chrome/reflective panels.
- Fresh typography and completely original high-quality XMB icon family,
  designed at touch and display sizes; verify pt-BR legibility.
- Polished light hierarchy in portrait/landscape, content status and depth.
- Independent preset shader tokens, reduced-effects/battery profile.

## Part 3 — full functional menu replacement

- Convert Mods, Packs, Shaders, Installed/Files, Settings, Skins, Account and
  Profiles into contextual XMB item lists and nested views, without deleting
  or obscuring pre-existing Horizon actions.
- Keep downloads, search, install/import, profile and shader operations wired.
- Contextual actions rather than shortcut cards; preserve application state,
  Back/return hierarchy, scrolling and Android IME flows.
- No default Material cards/pills in Old School flows.

## Part 4 — interaction and audiovisual feel

- Touch gesture engine with axis lock, fling, focus snap, touch-to-focus
  before activation, long-press context menus and Android back compatibility.
- Frame-rate-independent XMB kinetic navigation (60/90/120Hz), animation
  curves, ripple/halo cues, timing, reduced-motion compliance.
- Rebind the already packaged 66 SFX semantically; do not remove/replace files.
- Complete D-pad/controller, keyboard and accessibility focus behavior.

## Part 5 — audit, optimization, screenshots, APK

- Run native/Android CI and the appropriate automated test suites.
- Make actual native renderer preview PNGs for portrait/landscape, inspect
  typography, clipping, no-overlap and focus against acceptance criteria.
- Measure frame stability/idle cost; optimize shader drawing and allocation.
- Verify Liquid Glass baseline and Old School sound library have no regressions.
- Publish a testable ARM64 dev APK with build SHA and evidence, then report
  honestly what has and has not been tested on physical hardware.

Reference research:
- Sony PS3 XMB user guide:
  https://manuals.playstation.net/document/en/ps3/current/basicoperations/xmb.html
- Android edge-to-edge:
  https://developer.android.com/design/ui/mobile/guides/layout-and-content/edge-to-edge
- Android custom accessibility:
  https://developer.android.com/guide/topics/ui/accessibility/views/custom-views

Authored for this repository by OpenAI GPT-6 with the owner's instructions.
