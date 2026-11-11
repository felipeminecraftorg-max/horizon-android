# Android launcher appearance and interaction

The launcher has two optional material presets: **Liquid Glass** (cool violet
frosted surfaces) and **Dark Aero** (black glass and neutral silver). Open the
header's appearance button, or Settings > Launcher > Theme. Pick a preset and
then a wallpaper independently: Plain, Aurora, Graphite or Midnight. Glass
effects, interface sounds and launcher animations each have their own switch.
Advanced colors and the earlier palettes remain available.

The wallpaper art is original and procedural. Frosted surfaces sample a cached,
softened wallpaper, with a translucent tint and a cached nine-slice highlight.
This is a lightweight wallpaper material, not Apple's compositor or a live blur
of the game. The game's own visual and animation settings are independent.

Portrait navigation moves to the bottom; landscape uses a side rail. Home has
one primary launch button, the current profile and shortcuts to content. Status
details are behind a dismissible sheet rather than a long technical footer.

## Input and redraws

A pointer activates only the target it pressed, after releasing on that same
target without scrolling. Secondary fingers cannot finish the primary gesture.
Cancellation and lost focus discard it. Synthetic mouse clicks immediately
following a touch are ignored. Button sounds occur only after confirmation and
can be disabled for the launcher. Held buttons show immediate visual feedback.

The launcher redraws during input, transitions and asynchronous updates. At idle
it polls background results without continuously submitting frames. Catalog and
skin workers report changes so completed requests still appear. Text editing and
held controller navigation retain the active frame cadence.

## Native captures and validation

`crates/horizon-client/src/launcher_visual.rs` contains the shared shell, home and
appearance components used directly by Android. The capture tool uses those
same components, bundled fonts and the production `HudRenderer` GPU pipelines:

```sh
cargo run --locked -p horizon-client --no-default-features \
  --example launcher_capture -- /tmp/horizon-launcher-captures
```

It captures Home in landscape and Appearance in portrait for each new theme.
The account and installation state are fixture data; no real session or game
files are required. The second frame of each page verifies that warm UI textures
are reused without new atlas uploads. These captures verify native drawing,
not Android installation, touch hardware latency or phone FPS.

Validation also includes the gesture regression tests, native viewport and
configuration tests, the existing Android launcher source checks and an ARM64
release build. New appearance fields have defaults when reading older configs.
The compatibility work from dev 32 remains the base; no Java mod behaviour or
Create compatibility is added by this launcher change.
