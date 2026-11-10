//! Lane 2, block shapes: a mod block whose Java class makes its own shape (`getShape`,
//! MCreator's `makeShapes`) carries over with its template's (a full cube, a pumpkin's). The
//! model supplies an approximation: its element boxes, turned as its blockstate turns the
//! model. This cannot recover a Java-defined collision shape that differs from the artwork.
//!
//! Only blocks on a template whose shape is the block's whole (`minecraft:stone`, and the
//! facing ones `minecraft:carved_pumpkin` and `minecraft:furnace`), with a model of elements of
//! the mod's own.
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

/// The most boxes a shape has (the content API's limit).
const MAX_BOXES: usize = 16;
const FACINGS: [&str; 4] = ["north", "east", "south", "west"];

/// A Java class name's block id: `Level0PortalBlock` → `level_0_portal` (MCreator's and most
/// mods' naming), and the same with digits kept to their word (`level0_portal`).
pub fn block_ids(class: &str) -> Vec<String> {
    let name = class.strip_suffix("Block").unwrap_or(class);
    let mut words: Vec<String> = Vec::new();
    let mut previous: Option<char> = None;
    for c in name.chars() {
        let split = match previous {
            None => true,
            Some(p) => {
                (c.is_ascii_uppercase() && !p.is_ascii_uppercase())
                    || (c.is_ascii_digit() != p.is_ascii_digit())
            }
        };
        if split || words.is_empty() {
            words.push(String::new());
        }
        if let Some(w) = words.last_mut() {
            w.push(c.to_ascii_lowercase());
        }
        previous = Some(c);
    }
    let split = words.join("_");
    let mut joined = String::new();
    for (i, w) in words.iter().enumerate() {
        let digits = w.chars().all(|c| c.is_ascii_digit());
        if i > 0 && !digits {
            joined.push('_');
        }
        joined.push_str(w);
    }
    let mut out = vec![split];
    if !out.contains(&joined) {
        out.push(joined);
    }
    out
}

/// A model's element boxes in block units, following its parents in the mod's own models.
fn elements(carry: &Path, model: &str, depth: usize) -> Option<Vec<[f64; 6]>> {
    let (ns, path) = model.split_once(':').unwrap_or(("minecraft", model));
    if ns == "minecraft" || depth > 8 {
        return None;
    }
    let file = carry
        .join("assets")
        .join(ns)
        .join("models")
        .join(format!("{path}.json"));
    let v: Value = serde_json::from_slice(&fs::read(file).ok()?).ok()?;
    if let Some(list) = v["elements"].as_array() {
        let mut boxes = Vec::new();
        for e in list {
            // A tilted element is not its unrotated box. Leave this model to a native shape
            // implementation instead of writing an incorrect collision override.
            if !e["rotation"].is_null() && e["rotation"]["angle"].as_f64() != Some(0.0) {
                return None;
            }
            let corner = |key: &str| -> Option<[f64; 3]> {
                let a = e[key].as_array()?;
                Some([
                    a.first()?.as_f64()?,
                    a.get(1)?.as_f64()?,
                    a.get(2)?.as_f64()?,
                ])
            };
            let (from, to) = (corner("from")?, corner("to")?);
            let mut b = [0.0; 6];
            for i in 0..3 {
                let (lo, hi) = (from[i].min(to[i]), from[i].max(to[i]));
                // The content API accepts boxes beyond one block (e.g. an overhanging beam).
                b[i] = lo / 16.0;
                b[i + 3] = hi / 16.0;
            }
            // A flat plane (a cross plant's, a sign's face) holds nothing.
            if (0..3).all(|i| b[i + 3] - b[i] >= 1.0 / 64.0) {
                boxes.push(b);
            }
        }
        return Some(boxes);
    }
    let parent = v["parent"].as_str()?;
    elements(carry, parent, depth + 1)
}

/// At most `MAX_BOXES`: the largest, then one box around the rest.
fn fit(mut boxes: Vec<[f64; 6]>) -> Vec<[f64; 6]> {
    if boxes.len() <= MAX_BOXES {
        return boxes;
    }
    let volume = |b: &[f64; 6]| (b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2]);
    boxes.sort_by(|a, b| volume(b).total_cmp(&volume(a)));
    let rest = boxes.split_off(MAX_BOXES - 1);
    let mut around = rest[0];
    for b in &rest[1..] {
        for i in 0..3 {
            around[i] = around[i].min(b[i]);
            around[i + 3] = around[i + 3].max(b[i + 3]);
        }
    }
    boxes.push(around);
    boxes
}

/// A blockstate's negative XYZ quarter-turns, about the block centre, as in the renderer.
fn turn(boxes: &[[f64; 6]], rotations: [i64; 3]) -> Vec<[f64; 6]> {
    boxes
        .iter()
        .map(|b| {
            let corner = |mut p: [f64; 3]| {
                for (axis, angle) in rotations.into_iter().enumerate() {
                    let (a, c) = ((axis + 1) % 3, (axis + 2) % 3);
                    for _ in 0..angle.rem_euclid(360) / 90 {
                        let old = p[a];
                        p[a] = p[c];
                        p[c] = 1.0 - old;
                    }
                }
                p
            };
            // Quarter turns are signed axis permutations, so opposite corners suffice.
            let lo = corner([b[0], b[1], b[2]]);
            let hi = corner([b[3], b[4], b[5]]);
            std::array::from_fn(|i| {
                if i < 3 {
                    lo[i].min(hi[i])
                } else {
                    lo[i - 3].max(hi[i - 3])
                }
            })
        })
        .collect()
}

fn round(boxes: Vec<[f64; 6]>) -> Vec<[f64; 6]> {
    boxes
        .into_iter()
        .map(|b| b.map(|n| (n * 4096.0).round() / 4096.0))
        .collect()
}

/// A blockstate variant's model and turns. Invalid rotations must not become identity.
fn variant(v: &Value) -> Option<(&str, [i64; 3])> {
    let mut rotations = [0; 3];
    for (angle, key) in rotations.iter_mut().zip(["x", "y", "z"]) {
        *angle = if v[key].is_null() {
            0
        } else {
            v[key].as_i64()?
        };
        if angle.rem_euclid(90) != 0 {
            return None;
        }
    }
    Some((v["model"].as_str()?, rotations))
}

/// Random visual alternatives can share a collision only if every alternative agrees.
fn variant_shapes(carry: &Path, v: &Value) -> Option<Vec<[f64; 6]>> {
    let options = v
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or(std::slice::from_ref(v));
    let mut common = None;
    for option in options {
        let (model, rotations) = variant(option)?;
        let boxes = elements(carry, model, 0)?;
        if boxes.is_empty() {
            return None;
        }
        let mut boxes = round(fit(turn(&boxes, rotations)));
        boxes.sort_by(|a, b| {
            a.iter()
                .zip(b)
                .map(|(a, b)| a.total_cmp(b))
                .find(|c| !c.is_eq())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if common.as_ref().is_some_and(|previous| *previous != boxes) {
            return None;
        }
        common = Some(boxes);
    }
    common
}

/// The `shapes` of block `path` from its blockstate and models, for template property
/// `property` (`""` for a template with one state shape, `facing`).
fn shapes_of(carry: &Path, ns: &str, path: &str, property: &str) -> Option<Value> {
    let file = carry
        .join("assets")
        .join(ns)
        .join("blockstates")
        .join(format!("{path}.json"));
    let state: Value = serde_json::from_slice(&fs::read(file).ok()?).ok()?;
    let variants = state["variants"].as_object()?;
    let mut values = BTreeMap::new();
    if property.is_empty() {
        let mut common = None;
        for v in variants.values() {
            let boxes = variant_shapes(carry, v)?;
            if common.as_ref().is_some_and(|previous| *previous != boxes) {
                return None;
            }
            common = Some(boxes);
        }
        values.insert(String::new(), common?);
    } else {
        let mut by_facing = BTreeMap::new();
        for (key, v) in variants {
            let facing = key
                .split(',')
                .find_map(|kv| kv.strip_prefix(&format!("{property}=")));
            if let Some(f) = facing
                && let Some(&name) = FACINGS.iter().find(|x| **x == f)
            {
                let boxes = variant_shapes(carry, v)?;
                if by_facing
                    .get(name)
                    .is_some_and(|previous| *previous != boxes)
                {
                    return None;
                }
                by_facing.insert(name, boxes);
            }
        }
        let north = by_facing.get("north")?;
        for (i, f) in FACINGS.iter().enumerate() {
            let boxes = by_facing
                .get(*f)
                .cloned()
                .unwrap_or_else(|| round(turn(north, [0, 90 * i as i64, 0])));
            values.insert((*f).to_owned(), boxes);
        }
    }
    // A shape that is the whole block is the template's already.
    if values
        .values()
        .all(|b| b.len() == 1 && b[0] == [0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
    {
        return None;
    }
    Some(serde_json::json!({"property": property, "values": values}))
}

/// Gives the carried blocks of mod `id` (in `carry`) whose classes (`classes`, simple names)
/// make their own shapes the shapes of their models. Returns how many got one.
pub fn add_model_shapes(carry: &Path, id: &str, classes: &BTreeSet<String>) -> usize {
    let decls = carry.join("data").join(id).join("horizon/block");
    let mut done = 0;
    for class in classes {
        for path in block_ids(class) {
            let decl_file = decls.join(format!("{path}.json"));
            let Ok(bytes) = fs::read(&decl_file) else {
                continue;
            };
            let Ok(mut decl) = serde_json::from_slice::<Value>(&bytes) else {
                break;
            };
            if ["shapes", "thickness", "connects_to"]
                .iter()
                .any(|k| !decl[*k].is_null())
            {
                break;
            }
            let property = match decl["template"].as_str() {
                Some("minecraft:stone") => "",
                Some("minecraft:carved_pumpkin" | "minecraft:furnace") => "facing",
                _ => break,
            };
            if let Some(shapes) = shapes_of(carry, id, &path, property) {
                decl["shapes"] = shapes;
                if let Ok(text) = serde_json::to_vec_pretty(&decl)
                    && fs::write(&decl_file, text).is_ok()
                {
                    done += 1;
                }
            }
            break;
        }
    }
    done
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the actual carry-over entry point, including the block declaration it writes.
    fn inferred(variants: Value, models: &[(&str, Value)], template: &str) -> Option<Value> {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let carry = std::env::temp_dir().join(format!(
            "port-pack-shape-regression-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let write = |path: &str, value: &Value| {
            let file = carry.join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, serde_json::to_vec(value).unwrap()).unwrap();
        };
        write(
            "assets/ruby/blockstates/probe.json",
            &serde_json::json!({"variants": variants}),
        );
        for (name, model) in models {
            write(&format!("assets/ruby/models/block/{name}.json"), model);
        }
        write(
            "data/ruby/horizon/block/probe.json",
            &serde_json::json!({"template": template}),
        );
        let count = add_model_shapes(&carry, "ruby", &BTreeSet::from(["ProbeBlock".into()]));
        let declaration: Value = serde_json::from_slice(
            &fs::read(carry.join("data/ruby/horizon/block/probe.json")).unwrap(),
        )
        .unwrap();
        fs::remove_dir_all(carry).unwrap();
        assert_eq!(count, usize::from(declaration.get("shapes").is_some()));
        declaration.get("shapes").cloned()
    }

    fn model(from: [i32; 3], to: [i32; 3]) -> Value {
        serde_json::json!({"elements": [{"from": from, "to": to}]})
    }

    #[test]
    fn carried_collision_follows_xyz_model_rotations() {
        let models = [("probe", model([0, 0, 0], [4, 8, 12]))];
        // Quarter turns about the block centre, X then Y then Z, as in the renderer.
        for (rotation, expected) in [
            (
                serde_json::json!({"x": 90}),
                [0.0, 0.0, 0.5, 0.25, 0.75, 1.0],
            ),
            (
                serde_json::json!({"x": 90, "y": 90}),
                [0.0, 0.0, 0.0, 0.5, 0.75, 0.25],
            ),
            (
                serde_json::json!({"x": 90, "y": 90, "z": 90}),
                [0.0, 0.5, 0.0, 0.75, 1.0, 0.25],
            ),
        ] {
            let mut variant = rotation;
            variant["model"] = "ruby:block/probe".into();
            let shape =
                inferred(serde_json::json!({"": variant}), &models, "minecraft:stone").unwrap();
            assert_eq!(shape["values"][""], serde_json::json!([expected]));
        }
    }

    #[test]
    fn carried_collision_keeps_boxes_outside_the_block() {
        let shape = inferred(
            serde_json::json!({"": {"model": "ruby:block/probe"}}),
            &[("probe", model([-4, 0, 0], [4, 8, 20]))],
            "minecraft:stone",
        )
        .unwrap();
        assert_eq!(
            shape["values"][""],
            serde_json::json!([[-0.25, 0.0, 0.0, 0.25, 0.5, 1.25]])
        );
    }

    #[test]
    fn random_models_need_the_same_collision() {
        let variants = serde_json::json!({"": [
            {"model": "ruby:block/probe", "weight": 3},
            {"model": "ruby:block/other"}
        ]});
        let small = model([0, 0, 0], [4, 8, 12]);
        assert!(
            inferred(
                variants.clone(),
                &[
                    ("probe", small.clone()),
                    ("other", model([0, 0, 0], [8, 8, 12]))
                ],
                "minecraft:stone"
            )
            .is_none()
        );
        assert!(
            inferred(
                variants,
                &[("probe", small.clone()), ("other", small)],
                "minecraft:stone"
            )
            .is_some()
        );
    }

    #[test]
    fn a_template_cannot_collapse_different_state_shapes() {
        let models = [
            ("probe", model([0, 0, 0], [4, 8, 12])),
            ("other", model([0, 0, 0], [8, 8, 12])),
        ];
        for (template, first, second) in [
            ("minecraft:stone", "lit=false", "lit=true"),
            (
                "minecraft:furnace",
                "facing=north,lit=false",
                "facing=north,lit=true",
            ),
        ] {
            assert!(
                inferred(
                    serde_json::json!({(first): {"model": "ruby:block/probe"},
                    (second): {"model": "ruby:block/other"}}),
                    &models,
                    template
                )
                .is_none()
            );
        }
    }

    #[test]
    fn unsupported_rotations_do_not_write_an_unrotated_collision() {
        let mut tilted = model([0, 0, 0], [4, 8, 12]);
        tilted["elements"][0]["rotation"] = serde_json::json!({
            "origin": [8, 8, 8], "axis": "y", "angle": 45
        });
        assert!(
            inferred(
                serde_json::json!({"": {"model": "ruby:block/probe"}}),
                &[("probe", tilted)],
                "minecraft:stone"
            )
            .is_none()
        );
        assert!(
            inferred(
                serde_json::json!({"": {"model": "ruby:block/probe", "x": 45}}),
                &[("probe", model([0, 0, 0], [4, 8, 12]))],
                "minecraft:stone"
            )
            .is_none()
        );
    }

    #[test]
    fn class_names_become_block_ids() {
        assert_eq!(block_ids("AntennaBlock"), ["antenna"]);
        assert_eq!(
            block_ids("AlphaComputerPortalBlock"),
            ["alpha_computer_portal"]
        );
        assert_eq!(
            block_ids("Level0PortalBlock"),
            ["level_0_portal", "level0_portal"]
        );
        assert_eq!(
            block_ids("Asphalt2LinesWhiteBlock"),
            ["asphalt_2_lines_white", "asphalt2_lines_white"]
        );
    }

    #[test]
    fn a_turn_moves_the_north_side_clockwise() {
        // A slab against the north side.
        let north = [[0.0, 0.0, 0.0, 1.0, 1.0, 0.25]];
        assert_eq!(turn(&north, [0, 90, 0]), [[0.75, 0.0, 0.0, 1.0, 1.0, 1.0]]);
        assert_eq!(turn(&north, [0, 180, 0]), [[0.0, 0.0, 0.75, 1.0, 1.0, 1.0]]);
        assert_eq!(turn(&north, [0, 270, 0]), [[0.0, 0.0, 0.0, 0.25, 1.0, 1.0]]);
    }

    #[test]
    fn a_facing_block_takes_its_models_shapes() {
        let carry = std::env::temp_dir().join(format!("port-pack-shapes-{}", std::process::id()));
        let _ = fs::remove_dir_all(&carry);
        let write = |path: &str, text: &str| {
            let p = carry.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        };
        write(
            "assets/ruby/blockstates/antenna.json",
            r#"{"variants": {"facing=north": {"model": "ruby:block/antenna"},
            "facing=east": {"model": "ruby:block/antenna", "y": 90}}}"#,
        );
        write(
            "assets/ruby/models/block/antenna.json",
            r#"{"parent": "ruby:block/base"}"#,
        );
        write(
            "assets/ruby/models/block/base.json",
            r#"{"elements": [{"from": [6, 0, 0], "to": [10, 16, 4]},
            {"from": [0, 0, 8], "to": [16, 0, 16]}]}"#,
        );
        write(
            "data/ruby/horizon/block/antenna.json",
            r#"{"template": "minecraft:carved_pumpkin"}"#,
        );
        write(
            "data/ruby/horizon/block/flat.json",
            r#"{"template": "minecraft:oak_stairs"}"#,
        );
        let classes = BTreeSet::from(["AntennaBlock".to_owned(), "FlatBlock".to_owned()]);
        assert_eq!(add_model_shapes(&carry, "ruby", &classes), 1);
        let decl: Value = serde_json::from_slice(
            &fs::read(carry.join("data/ruby/horizon/block/antenna.json")).unwrap(),
        )
        .unwrap();
        let values = &decl["shapes"]["values"];
        assert_eq!(decl["shapes"]["property"], "facing");
        // The flat plane is left out; east is the blockstate's own turn, south and west north's.
        assert_eq!(
            values["north"],
            serde_json::json!([[0.375, 0.0, 0.0, 0.625, 1.0, 0.25]])
        );
        assert_eq!(
            values["east"],
            serde_json::json!([[0.75, 0.0, 0.375, 1.0, 1.0, 0.625]])
        );
        assert_eq!(
            values["south"],
            serde_json::json!([[0.375, 0.0, 0.75, 0.625, 1.0, 1.0]])
        );
        assert_eq!(
            values["west"],
            serde_json::json!([[0.0, 0.0, 0.375, 0.25, 1.0, 0.625]])
        );
        fs::remove_dir_all(&carry).unwrap();
    }
}
