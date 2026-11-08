# What Java mods need, and what Horizon Android supports

The Android mod converter (`crates/horizon-client/src/android/mod_convert.rs`) turns a Java
mod's jar or a modpack into Horizon content. What it can carry depends on what the engine
supports. This page lists what mods use, area by area, with Horizon's support today and the
lane that would cover what is missing:

- **Lane 1**, data: carried over by `horizon-port-pack`.
- **Lane 2**, behaviour as data: the engine implements a vanilla building block once (a mob
  type, a dimension, a block shape), and the converter fills it in from each mod's files or
  code constants.
- **Lane 3**, the mod's own Java code on Android's runtime, through Horizon's Java API slice
  (`packaging/android/java/`).
- **Lane 4**, a Horizon version of a known mod (`KNOWN`, `game/android_bundled/compat.rs`).

Status: **yes** (works), **partly**, **no**. The list comes from the converter's reports for
Krab's Modpack (adventure, Fabric 1.21.1) and Places (MCreator, NeoForge 1.21.1, 14
dimensions); each report's prompt names what its mods miss.

## World

| # | What mods use | Status | Lane | Notes |
|---|---|---|---|---|
| W1 | Their own dimensions (`data/<ns>/dimension/`, `dimension_type/`, noise settings, biomes) | no | 2 | The engine has three levels (`enum Dim`). Places has 14. |
| W2 | Portals into them (portal blocks, frames, igniters, teleport) | no | 2 | Needs W1. MCreator portals run `execute in <dim> run tp`. |
| W3 | Structures, template pools, structure sets, processors (data) | yes | 1 | YUNG's and Lithostitched handled. |
| W4 | Worldgen features and structure types written in Java | no | 2 | One Rust version per type. |
| W5 | Biome additions (Fabric `BiomeModifications` in code, NeoForge `biome_modifier` data) | partly | 1, 3 | Data ones carry. |
| W6 | Fluids of their own | no | 2 | |
| W7 | Painting variants, jukebox songs, enchantments, damage types (data) | yes | 1 | |

## Blocks

| # | What mods use | Status | Lane | Notes |
|---|---|---|---|---|
| B1 | Blocks on a vanilla template (full, slabs, stairs, walls, fences, doors, trapdoors, panes) | yes | 1 | Shape, hardness and sounds from the closest vanilla block. |
| B2 | Shapes defined in code (`makeShapes`, `Block.box`) | no | 2 | The content API has per-state shapes; the converter must read the constants. Places: 21. |
| B3 | Blocks with no vanilla template | partly | 2 | Places: 24 left out. |
| B4 | Use (right-click), random ticks, step on, entity inside, destroyed, exploded | no | 2, 3 | Native hook points exist for all of these. Places: 725 blocks. |
| B5 | Block entities: containers, machines, animated blocks | no | 3 | Mod API 0.4. Places: 15. |
| B6 | Sitting on blocks (chairs, benches) | no | 2 | Common in decoration mods. |
| B7 | Crops, saplings, plants of their own | partly | 1, 2 | Templates; growth on vanilla rules. |
| B8 | Render layers and glowing textures | partly | 1 | |

## Items

| # | What mods use | Status | Lane | Notes |
|---|---|---|---|---|
| I1 | Items on a vanilla template, stack size, rarity, components | yes | 1 | |
| I2 | Tools, armour, food through components | yes | 1 | |
| I3 | Using an item (right-click, on hit, on use over time) | no | 2, 3 | Places: 77. |
| I4 | Tooltip lines (`appendHoverText`) | no | 2 | Plain text could be data. |
| I5 | Armour and items with 3D or animated models (GeckoLib) | no | 2 | |
| I6 | Spawn eggs of their mobs | no | 2 | Needs E1. |
| I7 | A creative inventory tab | yes | 1 | The converter writes Horizon's tab (2026-11-08). |

## Entities

| # | What mods use | Status | Lane | Notes |
|---|---|---|---|---|
| E1 | Entity types of their own (registry, saving, the client knowing them) | partly | 2 | Only thrown projectiles (`snowball` template). |
| E2 | Mobs: attributes, AI from vanilla goals, natural spawns | no | 2 | Krab's: Creeper Overhaul, Living Things. Places: 55. |
| E3 | Mob models in Java code (`ModelPart`) | no | 2, 3 | |
| E4 | GeckoLib models and animations (data files) | no | 2 | One renderer serves every GeckoLib mod. |
| E5 | New looks for vanilla mobs (textures for the vanilla model) | yes | 2 | 2026-11-08, `mobs/variants.rs`. Mob Variants. |
| E6 | Projectiles other than thrown items, vehicles, mounts | no | 2 | |
| E7 | Bosses (boss bar, phases) | no | 3 | Boss bars exist in the engine. |

## Players and play

| # | What mods use | Status | Lane | Notes |
|---|---|---|---|---|
| P1 | Status effects that do something | partly | 2, 3 | The content API names them; effect hooks exist. |
| P2 | Combat changes (Better Combat), player animations (PlayerAnimator) | no | 4 | |
| P3 | Harvest, tree felling, vein mining | yes | 4 | 2026-11-08: Right Click Harvest, Tree Harvester, Veinminer. |
| P4 | Their own commands (Brigadier) | no | 3 | |
| P5 | Keys, HUD overlays | partly | 3 | Native mods have them; Java mods don't yet. |
| P6 | Recipe types of their own (machines) | no | 2, 3 | |
| P7 | Villager professions and trades | no | 2 | |
| P8 | Particle types of their own | no | 2 | Sounds work. |
| P9 | Container screens and their own screens | no | 3 | Places: 65 menus, 151 screens. |
| P10 | Network messages between their client and server code | no | 3 | Places: 31. |

## The Java side (lane 3)

| # | What mods use | Status | Notes |
|---|---|---|---|
| J1 | Fabric's loader and entrypoints | yes | Jar in jar libraries included. |
| J2 | Fabric API events | partly | Use block, break block, server ticks, lifecycle. |
| J3 | Minecraft's own classes | partly | A first slice; reports rank what mods stop at. |
| J4 | NeoForge's API (event bus, `DeferredRegister`) | no | Places, and every NeoForge mod. |
| J5 | Libraries: Architectury, Cloth Config, GeckoLib, Collective, owo-lib, Kotlin | partly | Kotlin comes in its mods' jars. |
| J6 | Mixins | no | Lane 4 ports, case by case. |
| J7 | MCreator's patterns (procedures that run commands) | no | Covers thousands of MCreator mods at once. |

## Order of work

1. The report says plainly when a mod has dimensions or portals (W1, W2).
2. Block shapes from code (B2).
3. Dimensions and portals (W1, W2).
4. MCreator procedures (J7) and block behaviour (B4).
5. Mod entity types and mobs (E1, E2), then GeckoLib (E4, I5).
6. NeoForge in lane 3 (J4), then menus and screens (B5, P9).
