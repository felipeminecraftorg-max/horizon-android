# Horizon Nexo — Preview Update 1

The approved five-panel 3D carousel is the source of this implementation. Geometry,
closed depth path, directional materials and the one-shot icon presentation remain
in `scene.js`. This is a new interface foundation, separate from the rejected Jade
layout and separate from the actual Horizon game launcher.

An exposed rear card selects; touching the front card opens that same surface.
The surface expands to fit the phone, the 3D icon travels into its header and
accessible native text/controls follow its projected face. Swiping never opens a
section. Back restores the selected card. There is no separate Enter control.

## Working in this preview

- Real mesh/depth rendering, reversible spring motion and pointer capture.
- Native-resolution front-card text follows the projected geometry independently
  of GPU render scale; module text uses the same sharp native layer.
- Mods/Packs/Shaders example catalogs with accent-insensitive local search,
  on/off/favorite filters, sorting, separate details/toggles and persistent state.
- Preview profile creation/selection, favorites, undo and pack priority controls.
- Real JSON catalog import/export with validation and atomic failure recovery.
- A real 3D demonstration character with independent touch rotation in Skins.
- Persistent text size, reduced motion, frame limit, GPU render scale, warm-light
  adjustment, original short procedural sound effects and optional native haptics.
- Android Back delegation, native pause/resume, on-demand rendering and a separate
  Android identity: `dev.horizon.nexo.preview`. Existing Horizon/Jade apps remain.

## Explicit integration limits

Catalog items are labeled local examples. Toggling one does not install or change
mods in Minecraft. Profiles are preview metadata. This APK has no account tokens,
Minecraft resources or game engine. The Play action reports the absent host;
it does not simulate a successful game launch. A host boundary `NexoNativeHost`
is defined for future launcher integration; its duplicate-launch guard is exercised
with a test boundary, not a real game instance.

Real game launch/session progress, account/profile synchronization, `.hmod`/ZIP/PNG
imports, real resource-pack priority, skin textures, verified compatibility and
filesystem deletion still require the Rust launcher integration. Full gamepad
support, drag handles for reordering and music/ambience assets are not completed.
Phone FPS, native IME/back interactions, cutouts, native sound/haptics and thermal
behavior must be verified on the Redmi; browser captures do not establish them.

## Validation and build

`node --test experiments/nexo-preview/tests/state.test.cjs` tests actual persistent
state/search/import/undo outcomes. `tests/browser-check.cjs` uses native browser
touch input, WebGL shader execution, real form/download operations and responsive
layouts. The Nexo workflow builds and verifies a signed Android 10+ APK and checks
all packaged assets against the source. Test signatures are temporary; future
preview upgrades signed by another run may require exporting the catalog and
reinstalling only Nexo Preview.

The existing offline Android scaffold is reused through Gradle source sets; the
small native changes preserve Jade's fallback and export directory. This does not
change the native Rust game or silently enable a new launcher preset.
