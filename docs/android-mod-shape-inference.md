# Collision inference in the Android mod converter

The converter's lane 2 can approximate a Java mod block's collision using its visual
model. This is a fallback, not execution of the original block's `getShape` or
`getCollisionShape` method. Models and collision shapes can intentionally differ.

Implementation: `crates/horizon-port-pack/src/shapes.rs`, called from
`crates/horizon-client/src/android/mod_convert.rs`.

## Supported cases

- Blocks identified by the class scan as defining their own shapes, with a matching
  carried block declaration and models in the mod's namespace.
- Full-cube templates (`minecraft:stone`) and horizontal-facing templates
  (`minecraft:carved_pumpkin`, `minecraft:furnace`).
- Cuboid model elements and parents within the mod's carried assets.
- Blockstate rotations in X, Y and Z, in multiples of 90 degrees, in the renderer's
  order, about the centre of the block.
- Element bounds outside the unit block, preserved rather than clamped.
- Random model alternatives whose inferred boxes agree. Different textures are fine.
- Additional state selectors only when their inferred shapes agree for each
  property value retained by the template.

Horizontal directions absent from the blockstate retain the existing fallback:
rotate the north collision to fill the template's other directions.

## Limits

The shape format permits at most 16 boxes. The existing approximation keeps the
15 largest boxes and encloses the rest in one box. This can fill gaps that were
empty in the original model. Very thin and flat elements are excluded by the
existing minimum-thickness rule. Coordinates are rounded to 1/4096 block.

The converter leaves the declaration's existing shapes alone. When inference
cannot represent a model or its variants, it writes no shape override; the block
keeps its template's collision. That fallback is not proof of correct collision.
Examples include tilted individual elements, multipart blockstates, invalid
rotations, missing parents, and conflicting state or random-variant shapes.

Exact support for those cases requires an explicit collision declaration, a
supported behaviour implementation, or a verified port. This helper does not
add machines, entities, HUD hooks, Java mixin execution, or Create compatibility.

## Validation

Run `cargo test --locked -p horizon-port-pack` and
`cargo clippy --locked -p horizon-port-pack --all-targets -- -D warnings`.

The regression tests call the real `add_model_shapes` entry point and inspect the
written declaration. They cover XYZ rotations, overhanging geometry, random
alternatives, states that a template cannot represent, and unsupported rotations.
They do not replace an Android gameplay comparison against the original mod.
