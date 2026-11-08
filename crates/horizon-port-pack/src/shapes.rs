//! Lane 2, block shapes: a mod block whose Java class makes its own shape (`getShape`,
//! MCreator's `makeShapes`) carries over with its template's (a full cube, a pumpkin's). The
//! shape a mod draws is almost always its model's: the boxes of the model's elements, turned as
//! its blockstate turns the model. So the carried block gets those as its `shapes`.
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
            let corner = |key: &str| -> Option<[f64; 3]> {
                let a = e[key].as_array()?;
                Some([
                    a.first()?.as_f64()?,
                    a.get(1)?.as_f64()?,
                    a.get(2)?.as_f64()?,
                ])
            };
            let (Some(from), Some(to)) = (corner("from"), corner("to")) else {
                continue;
            };
            let mut b = [0.0; 6];
            for i in 0..3 {
                let (lo, hi) = (from[i].min(to[i]), from[i].max(to[i]));
                b[i] = (lo / 16.0).clamp(0.0, 1.0);
                b[i + 3] = (hi / 16.0).clamp(0.0, 1.0);
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

/// Boxes turned clockwise (seen from above) by `y` degrees, as a blockstate turns a model.
fn turn(boxes: &[[f64; 6]], y: i64) -> Vec<[f64; 6]> {
    boxes
        .iter()
        .map(|b| {
            let corner = |x: f64, z: f64| match y.rem_euclid(360) {
                90 => (1.0 - z, x),
                180 => (1.0 - x, 1.0 - z),
                270 => (z, 1.0 - x),
                _ => (x, z),
            };
            let (x0, z0) = corner(b[0], b[2]);
            let (x1, z1) = corner(b[3], b[5]);
            [x0.min(x1), b[1], z0.min(z1), x0.max(x1), b[4], z0.max(z1)]
        })
        .collect()
}

fn round(boxes: Vec<[f64; 6]>) -> Vec<[f64; 6]> {
    boxes
        .into_iter()
        .map(|b| b.map(|n| (n * 4096.0).round() / 4096.0))
        .collect()
}

/// A blockstate variant's model and turn.
fn variant(v: &Value) -> Option<(String, i64)> {
    let v = v.as_array().and_then(|a| a.first()).unwrap_or(v);
    Some((
        v["model"].as_str()?.to_owned(),
        v["y"].as_i64().unwrap_or(0),
    ))
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
    let shape = |(model, y): (String, i64)| -> Option<Vec<[f64; 6]>> {
        let boxes = elements(carry, &model, 0)?;
        (!boxes.is_empty()).then(|| round(fit(turn(&boxes, y))))
    };
    let mut values = BTreeMap::new();
    if property.is_empty() {
        let first = variants.get("").or_else(|| variants.values().next())?;
        values.insert(String::new(), shape(variant(first)?)?);
    } else {
        let mut by_facing: BTreeMap<&str, (String, i64)> = BTreeMap::new();
        for (key, v) in variants {
            let facing = key
                .split(',')
                .find_map(|kv| kv.strip_prefix(&format!("{property}=")));
            if let (Some(f), Some(m)) = (facing, variant(v))
                && let Some(name) = FACINGS.iter().find(|x| **x == f)
            {
                by_facing.entry(name).or_insert(m);
            }
        }
        let (north_model, north_y) = by_facing.get("north").cloned()?;
        for (i, f) in FACINGS.iter().enumerate() {
            let m = by_facing
                .get(f)
                .cloned()
                .unwrap_or_else(|| (north_model.clone(), north_y + 90 * i as i64));
            values.insert((*f).to_owned(), shape(m)?);
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
        assert_eq!(turn(&north, 90), [[0.75, 0.0, 0.0, 1.0, 1.0, 1.0]]);
        assert_eq!(turn(&north, 180), [[0.0, 0.0, 0.75, 1.0, 1.0, 1.0]]);
        assert_eq!(turn(&north, 270), [[0.0, 0.0, 0.0, 0.25, 1.0, 1.0]]);
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
