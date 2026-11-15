# Horizon Nexo — Preview Update 5 — spacious libraries and a Nexo introduction

The approved five-panel 3D carousel is the source of this implementation. Geometry,
closed depth path, directional lighting and the one-shot icon presentation remain
in `scene.js`. This is a new interface foundation, separate from the rejected Jade
layout and separate from the actual Horizon game launcher.

An exposed rear card selects; touching the front card opens that same surface.
There is no separate navigation strip: the actual panels are the navigation.
The surface expands to fit the phone, the 3D icon travels into its header and
accessible native text/controls follow its projected face. Swiping never opens a
section. A return arrow inside the panel or a tap on the outside background
restores the selected card. Outside drags and list scrolling never dismiss it.
The return keeps its reverse animation and fades the content with the same
surface. There is no separate Enter control.

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
  adjustment, seven softly mastered CC0 cues and optional native haptics.
- Android Back delegation, native pause/resume, suspended-background rendering and a separate
  Android identity: `dev.horizon.nexo.preview`. Existing Horizon/Jade apps remain.

## Update 2 visual finish

The same scene/controller now has original stylized chrome and smoke-polymer
materials: analytic reflected softboxes, a Fresnel response, a clear-coat lobe,
mipmapped seeded microtexture, thin machined edges and actual beveled prism icons.
The implementation is a lightweight artistic material model. Update 3 adds an
actual geometry shadow map with four filtered depth comparisons and a clipped
planar reflection pass for the selected panel. The reflection renders the same
mesh commands from a mirrored view and uses a small frosted filter. It is not
recursive reflection, refraction, ray tracing or full physically based transport.

Three layered shader ribbons flow behind the carousel. The same monotonic
animation clock moves the material lighting. Idle atmosphere renders at no more
than 24 FPS; gestures and transitions retain the chosen 30/45/60 cap. Android
pause/hidden state cancels both timer and animation frame. Reduced motion and the
independent live-background switch freeze ambient animation, while moving a card
still changes its reflection through its actual surface orientation.

Michroma is bundled offline for the main display titles; body text remains a
readable system sans. Its SIL OFL 1.1 license and copyright are bundled as
`android/src/main/assets/OFL-Michroma.txt`. See `THIRD_PARTY_NOTICES.md` for the
pinned source. No Sony/Microsoft logos, proprietary typefaces or console assets
are included. The existing preview state migrates without replacing the catalog.

## Update 3 navigation, background and audio

The WebGL background now covers the entire viewport, including behind the header
and footer. A separate viewport inside that same renderer keeps the actual
carousel and open panels within the safe content area. Screen hit-testing,
projected native text and the 3D header icon use the corresponding full-screen
coordinates. No duplicated background canvas or CSS-height crop is used.

Only the physical cards select/open sections. Invisible native button targets
follow those same projected faces for keyboard/screen-reader activation and
focus; there is no secondary row or global arrow-key menu. The integrated return
arrow is a 48-pixel target in the panel header. Android Back is retained.

The seven audio cues are edited Kenney Interface Sounds 1.0 assets (CC0),
from five upstream files. The sharp tonal cues and old error buzz are replaced by
short, lower/filtered samples with 6 ms attacks, soft tails and matched levels.
Decoded peaks remain below -10 dBFS. Playback has another 0.5 master factor,
per-event gains, a single active voice and 110/180 ms repeat limits. Generic
dialog open/close no longer plays an extra cue. Source hashes and exact rate,
EQ, trim, fade and gain settings are bundled in `sounds/sources.json`.

Motion blur uses current/previous per-mesh transforms and camera projection to
write a real per-pixel screen-velocity buffer. A 1/90-second shutter factor is
independent of the render FPS; vectors are capped at 32 framebuffer pixels. Nine
samples follow that pixel's motion with depth-aware edge dilation. Only the
moving region is composited. Static backgrounds/header/controls keep their
original image; at rest the extra GPU pass is skipped. Reduced motion disables
it, and viewport resize/native pause resets history. The native front-card title
uses its corresponding GPU text while its panel moves and returns to sharp
native text at rest. There is no CSS Gaussian blur on the scene or controls.

Shadow comparisons now use high-precision fragment arithmetic and back-face
casters, with a wider bias to suppress self-shadow speckling. The shadow fill is
softer. Maps reuse stationary geometry and reflected lighting updates at up to
12 FPS while still. Actual meshes still cast the shadows and provide the planar
reflection in the selected panel.

## Update 5 library and introduction

The home utility controls are a compact profile lens and settings dial in one
quiet material rail. Their native actions and accessible names remain. During
panel opening, that rail/footer fade and the renderer interpolates its safe
viewport toward the usable screen. The expanded panel keeps an outer margin,
cutout insets, its own integrated return and contextual system actions.

Search stays in one toolbar; filtering/sorting move into a contextual dialog.
The result count lives in the header, and rows use integrated toggle rails.
Portrait has one column; landscape has two. Rows keep readable type and 48px
controls. Scroll restore waits for the finished expansion and scroll anchoring
is disabled so the first entry is not cut by late layout changes. Browser
capacity checks import a 72-entry test-only manifest, not extra shipped mods.

The original 1.85-second intro draws the Nexo mark with vector light traces,
reveals the wordmark, hands it toward the home brand and brings in the actual
3D carousel from depth. It shares the live scene, is silent, can be skipped or
replayed, and respects reduced motion plus a saved startup switch. Native pause
freezes both animation timing and scene rendering; resuming does not restart it.
Replay from an open module preserves that module and fades back to its content.
No external movie, generated raster asset or new audio is required.

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
