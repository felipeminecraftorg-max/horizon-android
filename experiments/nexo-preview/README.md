# Horizon Nexo — Preview Update 3 — direct tabs and sampled audio

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
  adjustment, seven bundled CC0 audio recordings and optional native haptics.
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

The oscillator cues are replaced by seven original, unmodified Kenney recordings
(CC0), from Interface Sounds 1.0 and UI Audio. Selection, expansion, return,
confirmation, toggles, details and errors have separate short cues. A local
HTMLAudio player preloads the files, respects the saved volume, limits repeated
cues and stops all playback on native pause/hidden state. There is no generated
fallback tone or remote request. Pack notices and exact provenance/checksums are
bundled in `android/src/main/assets/sounds/`; see THIRD_PARTY_NOTICES.md.

Transitions now use velocity-dependent blur, with a small blur during panel
expansion/collapse; the native text layer receives a matching screen-space blur.
The blur settles to zero and reduced motion disables it. Shadows and the selected
panel's planar reflections can be switched off together in settings. Their map
resolution follows the existing render-scale setting; native pause cancels all
render passes. Shadow and reflection maps reuse the actual scene meshes.

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
