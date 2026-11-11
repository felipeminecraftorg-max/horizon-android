# Android launcher appearance and interaction

The launcher has two optional material presets: **Liquid Glass** (cool violet
frosted surfaces) and **Dark Aero** (black glass and neutral silver). Open the
header's appearance button, or Settings > Launcher > Theme. Pick a preset and
then a wallpaper independently: Plain, Aurora, Graphite or Midnight. Glass
effects, interface sounds and launcher animations each have their own switch.
Advanced colors and the earlier palettes remain available.

The wallpaper art is original and procedural. Liquid Glass now reads the already
drawn launcher scene beneath each surface. A rounded-rectangle distance field
defines its convex normal; Snell refraction bends the background, a nine-tap
filter softens it, and a narrow directional highlight and restrained chromatic
edge finish the lens. Text and icons are drawn afterwards and remain sharp.
The default tint opacity is 14%, instead of the earlier 56% grey overlay.

With Liquid Glass and effects enabled, scroll down in Appearance for three tap
controls: Transparency (70%, 86%, 92%), Backdrop blur (off, soft, strong) and
Refraction (off, natural, strong). These settings persist and old configurations
receive defaults. Turning blur off uses one background sample rather than nine;
turning Glass effects off removes all backdrop targets and passes.

The compositor reuses two screen-sized GPU textures, copies only each visible
lens's region plus its sampling halo, and composites in drawing order. It never
reads the screen back to the CPU during normal use and does not continuously
redraw an idle launcher. This adds GPU bandwidth and texture memory when enabled;
phone performance must be measured on the device. Dark Aero retains its cached
wallpaper sheen. This is an original approximation, not Apple's compositor.
The game's own visuals and animation settings are independent.

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

The capture tool also runs numerical GPU checks against a changing striped
background: refraction bends it, blur lowers its contrast, the centre remains
transparent, foreground UI stays sharp, and a scroll clip excludes the lens.
These checks use the production compositor, not a separate shader substitute.

Validation also includes the gesture regression tests, native viewport and
configuration tests, the existing Android launcher source checks and an ARM64
release build. New appearance fields have defaults when reading older configs.
The compatibility work from dev 32 remains the base; no Java mod behaviour or
Create compatibility is added by this launcher change.
