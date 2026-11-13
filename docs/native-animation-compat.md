# Horizon native animation compatibility — Build 46 development

Base commit: 2914d6cc9b1e4f093ef933dd5ecc1a05c72c8b74.
Working branch: feature/build46-emf-etf-core. The original Build 46 is not altered.

## What landed in the real Horizon workspace

The existing horizon-optifine crate now exposes these independently testable APIs:

- entity_models::parse and parse_with: parse bounded OptiFine CEM .jem models,
  plus .jpm parts resolved by a caller-supplied resource pack loader. Read
  geometry boxes, UV offsets, rotations, translations, child parts and expression
  source. Reject traversal references, oversized models and excessive nesting.
- entity_textures::parse and TextureRules::select: parse Java properties through
  Horizon's existing Properties type, then select stable per-entity skin/texture
  variants with weights and supported name, biome and height predicates.
  Unsupported rule predicates disable the entire rule (not simply the predicate).
- entity_animations::Compiled and Animation: compile bounded expression ASTs once
  during resource loading, then sample with game-supplied render variables.
  Includes arithmetic, conditions, comparisons, trigonometry, basic math and
  lazily evaluated if.

- entity_resources::EntityResources::load: scan enabled merged resource-pack
  overlays for CEM/JPM and supported ETF .properties files, with size and
  recursion limits. Compile valid animations once, keep warnings bounded and
  expose O(log n) lookups by namespaced entity/texture. Files are loaded from
  the game resource overlay; JAR code is never executed.
- entity_pose::Pose: compute local and parent-relative model part matrices,
  including animation channels for rotation, translation, scale and visibility.
  PreparedModel::pose samples animations against game-supplied variables.
- horizon-client-render: RenderAssets now retains the prepared entity resource
  registry during reloads, rather than discarding it with the block-model
  scratch table. This is a CPU integration only, not a GPU render override.

Validation: cargo test --locked -p horizon-optifine
Renderer compile: cargo check --locked -p horizon-client-render


## Explicit limitations — not yet a functional Fresh Animations port

These APIs prepare resource data but do not replace entity models or textures
in the renderer yet. There is no renderer binding, GPU upload, living entity
pose input mapping, texture atlas wiring, or texture emittance/blinking support.

This is a strict subset of EMF 3.3.12 and ETF 7.2.7 functionality:
- EMF CEM complex aliases, some JPM/face-UV semantics, custom textures,
  specialized animation functions and per-entity model variants remain missing.
- ETF unsupported predicates, emissive layers, blinking, texture overrides and
  full NBT metadata remain missing. Unsupported random texture rules are skipped.
- ReAnimated and Cinematic Interactions are NOT integrated by this branch.
- Fresh Animations was not supplied as a resource pack, so it has not been
  tested against the complete real pack.
- Neither ARM64 APK installation nor in-game visual parity is established.

## Next integration boundary

The game resource overlay is in horizon-client-render/src/packs.rs and the
entity geometry is submitted through the horizon-client-render graphics system.
The next step must emit GPU geometry/texture bindings from the already loaded
CEM data and evaluated part matrices, updating only a presentation pose (not server gameplay or hit testing), then
bind ETF texture variants to the actual texture atlas. Guard that integration
behind a disabled-by-default setting until visual comparisons with vanilla
and Fresh Animations succeed. Keep ReAnimated and Cinematic Interactions in
separate branches/modules to minimize regressions.

Java .jar mods from Fabric/NeoForge cannot be installed as native plugins
without porting. The current code does not claim otherwise.
