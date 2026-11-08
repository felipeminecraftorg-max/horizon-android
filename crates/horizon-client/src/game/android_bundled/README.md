# Fixed Android mod sources

The parent `android_bundled.rs` wires a fixed, opt-in set of MIT native modules
into existing game hook points. It is not a native-mod loader or compiler.
`PROVENANCE.json` pins source revisions; every module retains its MIT LICENSE.

The runtime and adapter sources come from the corresponding upstream mod.
Helper files are renamed and expose `run` for static calls. Original
`horizon-mixin.toml` files are retained for reference, not used for building:
this directory is not a directly installable `.hnmod` bundle.

Android changes reset profile-scoped state on launcher transitions, retain the
Android debug text scale and preserve the code-of-conduct screen. Update the
portable runtime tests and Android checks when changing these adapters.

`compat.rs` is not from an upstream mod: it is Horizon Android's own version of
what three Java mods' code does (Right Click Harvest, Tree Harvester,
Veinminer), written from their behaviour. The mod converter
(`android/mod_convert.rs`, `KNOWN`) turns one on when a converted pack has the
Java mod; like the others, each is off until turned on.

`sound_physics.rs` is Horizon Android's own version of Sound Physics Remastered,
written from what the mod does, not from its code: the window measures each new
world sound against its copy of the world (the blocks between the sound and the
listener, rays bounced around the sound) and the mixer
(`horizon_client::audio::Physics`) muffles it and feeds four echoes. Converting
the mod's jar turns it on.

`camera_overhaul.rs` is its own version of CameraOverhaul (camera roll and pitch
from movement and turning, idle sway, screen shakes), and the first-person body
of First-person Model is drawn by `game/perspective.rs`. `bundled_settings.rs`
gives these ports their settings pages in Options > Mods (a button per setting
that steps through its values).

`lane3.rs` starts the Java mods the converter made ready (lane 3: their own
code, run by Android's runtime; the Java half is
`packaging/android/java/dev/horizon/android/lane3/`), fires the Fabric events
Horizon has hooks for, and answers the `Native` methods of Horizon's Java API.
