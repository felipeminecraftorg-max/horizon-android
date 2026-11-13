# Native Horizon: Four-mod test matrix (Build 46 base)

This draft branch aims to provide a single testable client with native
adaptations of **ReAnimated**, **Cinematic Interactions**, **Entity Model
Features (EMF)** and **Entity Texture Features (ETF)**. The external
**Fresh Animations** pack is a set of resources to be verified against EMF/ETF,
not a fifth Java runtime mod.

This is NOT complete parity with the supplied Java JARs and is NOT an
Android release. No Java classes or Fabric/NeoForge Mixins are loaded at runtime.

## Native code that has been integrated

| Mod | Actual code | Current boundary |
| --- | --- | --- |
| ReAnimated 26.3+1.8 | `game/reanimated.rs`, `Game::hud()` | Frame-rate-independent screen entrance and opacity (scale is disabled on Android to keep touch alignment); screen ID transitions, reduced-motion configuration, unit tests |
| Cinematic Interactions 0.18.30 | `game/cinematic_interactions.rs`, `Game::render()` | First-person menu-triggered camera spring for crafting, enchanting, anvil, stonecutter, smithing, grinding, cartography, loom, beacon; additive over existing camera hooks, preserves original directional audio listener |
| EMF 3.3.12 | `horizon-optifine/entity_models.rs`, `entity_pose.rs`, `entity_animations.rs`, `client-render/cem_mesh.rs` | Bounded CEM/JPM parser, expressions, CPU mesh generation, experimental zombie/creeper model previews |
| ETF 7.2.7 | `horizon-optifine/entity_textures.rs`, `entity_resources.rs`, HUD atlas preview | Partial rule matching and weighted entity variants; optional atlas pack discovery with 256-image cap |

## Trial switches (developer-only; OFF by default)

In the prepared game resource root, create these *local* UTF-8 files with
exactly `enabled` as their content, then restart:

- `horizon-reanimated-preview.flag` — enable screen entrance motion.
- `horizon-cinematic-preview.flag` — enable workstation camera transitions.
- `horizon-cem-preview.flag` — allow limited CEM/ETF model and atlas previews.

Alternatively set process environment variables
`HORIZON_REANIMATED_PREVIEW=1`,
`HORIZON_CINEMATIC_PREVIEW=1`,
`HORIZON_CEM_PREVIEW=1` before the game starts.
The switches are **not** toggles in the Android launcher UI yet.
**Do not enable in an important world** while visual parity is unproven.
Since the baseline is disabled, normal releases retain the original visuals.

### Optional local animation configuration (no rebuild)

Besides the individual preview flags, the game reads a bounded 16 KiB
`horizon-native-animations.json` from the same prepared resource directory
at startup. For example:

```json
{
  "reanimated": {
    "enabled": true,
    "speed": 1.0,
    "buttons": true,
    "reduced_motion": false
  },
  "cinematic": {
    "enabled": true,
    "speed": 1.0,
    "roll_strength": 1.0,
    "reduced_motion": false
  }
}
```

This file is **not** loaded from a multiplayer resource pack and does not
enable EMF/ETF CEM rendering; that still uses the separate, default-off CEM
preview flag. There is no GUI configuration editor yet. Invalid/oversized
JSON falls back to disabled defaults. The values are parsed and clamped.
ReAnimated now animates screen entry and draws lightweight hover/press
decorations around buttons, tabs, sliders, list entries and inventory slots,
without moving touch or controller targets. The animations use the
last-published frame widget rectangles, and the original button input handlers
remain in charge of all clicks.

Cinematic Interactions now also animates first-person *hand presentation*
without changing the FirstPerson Model path, plus a short camera accent
after valid workstation menu actions. These effects are visual only; sound
orientation, reach, block interaction and inventory rules remain untouched.

### Smoke test sequence on a debug build

1. Check title, pause, options, inventory and crafting GUI for animation,
   readable text and no displaced Android touch targets.
2. Open/close crafting, enchanting, anvil, smithing and stonecutting. Observe
   spring smoothness, return-to-vanilla view and camera movement in first person.
3. Check movement/camera without a station active; audio directions must not change
   when the cinematic preview is toggled.
4. Test simple CEM models for zombies/creepers; then introduce invalid .jem/.jpm
   files, missing texture variants and disabled previews to confirm vanilla
   fallback. Full Fresh Animations is not expected to work yet.
5. Verify frame time (60/90/120 Hz), device memory and app switching/resume.
6. Check FirstPerson Model, camera controls and the pre-existing Sound Physics
   directional/stereo correction for regressions.
7. Test server play and inventory transactions to ensure no scene logic changes
   game physics, clicks or networking.

## Original JAR parity — missing features

### ReAnimated
The Fabric JAR has 22 listed client Mixins and settings for screen open/close,
pause, container and list transitions, tooltips, toasts, widgets, tab switching,
recipe book, UI logo, hover, pressed states and custom per-screen profiles.
Current native integration covers entrances, hover/press decorations and several widget categories. It does not yet reproduce every original animation or screen profile.
The remaining widgets must be animated in the native UI drawing code, using
the corresponding hitbox bounds and without moving touch interactions.

### Cinematic Interactions
The original has many Fabric/server and client Mixins, including camera,
first-person hands, block entities, station-specific scene rendering, GUI
remapping, sound, reaching, client/server station coordination and workstation
animation. The current native implementation is a camera and first-person-hand
presentation layer; it does not yet draw interactive 3-D workstation scenes or
reimplement network-synced crafting/enchanting/brewing actions. Any such
features need feature parity and multiplayer testing before they can ship.

### EMF/ETF
Many CEM model attributes, expressions and resource aliases remain unsupported.
ETF does not yet include blinking, emissive render passes, full NBT predicates
or all random entity texture conventions. Do not call these complete.
Fresh Animations itself must be tested with pack copyright respected and
without redistributing the resource pack unless licensed.

## Validation

```sh
cargo test --locked -p horizon-optifine
cargo check --locked -p horizon-client-render
cargo check --locked -p horizon-client --no-default-features
cargo check --locked -p horizon-client --no-default-features --tests
```

The CI workflow `.github/workflows/native-animation-compat.yml`
runs these steps. These checks do not prove Android ARM64 APK installation or
on-device visual parity. Keep PR #17 draft until those checks are complete.
