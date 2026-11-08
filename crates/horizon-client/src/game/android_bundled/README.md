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
