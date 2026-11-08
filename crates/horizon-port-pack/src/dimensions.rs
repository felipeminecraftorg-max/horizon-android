//! A mod's own dimensions, carried over: its level stems (`dimension/`), their dimension types,
//! and the biomes, noise, density functions and carvers they name, rewritten from the mod's
//! version in 26.3's formats so the game generates them as the mod defines them. The game reads
//! them as a data pack below the world's (`datapack::mod_pack`), and `/execute in <id>` reaches
//! them; the portals into them are the mod's Java code.
//!
//! A 1.21.1 mod writes these as 26.2 does, with two more differences 26.2 already had: a biome's
//! sky, fog and underwater fog colours were `effects` (26.2: `attributes`), and its carvers were
//! listed by carving step (26.2: one list). The rest is [`crate::upgrade`]'s 26.2 to 26.3.
use serde_json::{Map, Value, json};
use std::{collections::BTreeMap, fs, path::Path};

/// The folders of a dimension's worldgen, under `data/<namespace>/`.
const KINDS: [&str; 7] = [
    "dimension",
    "dimension_type",
    "worldgen/biome",
    "worldgen/configured_carver",
    "worldgen/density_function",
    "worldgen/noise",
    "worldgen/noise_settings",
];

/// What carrying a mod's dimensions did.
#[derive(Debug, Default)]
pub struct Report {
    /// Its level stems.
    pub dimensions: Vec<String>,
    /// Files rewritten in 26.3's formats.
    pub upgraded: usize,
    /// What has no faithful 26.3 form, with how often.
    pub lossy: BTreeMap<String, usize>,
}

/// Every `.json` under `dir`, by its path relative to `root` (`/`-separated).
fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let Ok(read) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out)?;
        } else if path.extension().is_some_and(|e| e == "json")
            && let Ok(rel) = path.strip_prefix(root)
        {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.insert(rel.to_string_lossy().replace('\\', "/"), bytes);
        }
    }
    Ok(())
}

/// Whether the files are written in a version before 26.3: noise settings with their surface
/// rule (26.3 names a material rule), a biome with its spawners or carvers by step, a
/// configured carver.
fn before_26_3(files: &BTreeMap<String, Value>) -> bool {
    files.iter().any(|(path, v)| {
        path.contains("/worldgen/configured_carver/")
            || v.get("surface_rule").is_some()
            || v.pointer("/generator/settings/surface_rule").is_some()
            || (path.contains("/worldgen/biome/")
                && (v.get("spawners").is_some() || v.get("carvers").is_some_and(Value::is_object)))
    })
}

/// A 1.21.1 biome as 26.2 writes it: its sky, fog and underwater fog colours as attributes, its
/// carvers as one list. A carver step that names a tag is left out, counted in `lossy`.
pub fn biome_26_2(v: &mut Value, lossy: &mut BTreeMap<String, usize>) {
    let Some(o) = v.as_object_mut() else {
        return;
    };
    let mut moved = Map::new();
    if let Some(effects) = o.get_mut("effects").and_then(Value::as_object_mut) {
        for (old, new) in [
            ("sky_color", "minecraft:visual/sky_color"),
            ("fog_color", "minecraft:visual/fog_color"),
            ("water_fog_color", "minecraft:visual/water_fog_color"),
        ] {
            if let Some(c) = effects.remove(old) {
                moved.insert(new.to_owned(), colour(&c));
            }
        }
    }
    if !moved.is_empty() {
        let attributes = o.entry("attributes").or_insert_with(|| json!({}));
        if let Some(a) = attributes.as_object_mut() {
            for (k, c) in moved {
                a.entry(k).or_insert(c);
            }
        }
    }
    if let Some(Value::Object(steps)) = o.get("carvers") {
        let mut list = Vec::new();
        for step in steps.values() {
            match step {
                Value::String(s) if s.starts_with('#') => {
                    *lossy.entry("biome: a tag of carvers".into()).or_default() += 1
                }
                Value::String(_) | Value::Object(_) => list.push(step.clone()),
                Value::Array(a) => list.extend(a.iter().cloned()),
                _ => {}
            }
        }
        o.insert("carvers".into(), Value::Array(list));
    }
}

/// A colour as a `#rrggbb` string: 1.21.1 wrote them as signed ints with the alpha bits set.
fn colour(c: &Value) -> Value {
    match c.as_i64() {
        Some(n) => Value::String(format!("#{:06x}", (n as u32) & 0xFF_FFFF)),
        None => c.clone(),
    }
}

/// Upgrades the carried dimensions of `inside` (the carry's folder) in the namespaces `owned`
/// in place. Nothing changes for a mod already in 26.3's formats.
pub fn carry(inside: &Path, owned: &[&str]) -> Result<Report, String> {
    let mut report = Report::default();
    let mut raw = BTreeMap::new();
    for ns in owned.iter().filter(|ns| **ns != "minecraft") {
        for kind in KINDS {
            walk(inside, &inside.join(format!("data/{ns}/{kind}")), &mut raw)?;
        }
    }
    for path in raw.keys() {
        if let Some(id) = path
            .strip_prefix("data/")
            .and_then(|r| r.split_once("/dimension/"))
            .and_then(|(ns, p)| Some(format!("{ns}:{}", p.strip_suffix(".json")?)))
        {
            report.dimensions.push(id);
        }
    }
    if report.dimensions.is_empty() {
        return Ok(report);
    }
    let mut json: BTreeMap<String, Value> = BTreeMap::new();
    for (path, bytes) in &raw {
        if let Ok(v) = serde_json::from_slice::<Value>(bytes) {
            json.insert(path.clone(), v);
        }
    }
    if !before_26_3(&json) {
        return Ok(report);
    }
    let mut files = BTreeMap::new();
    for (path, mut v) in json {
        if path.contains("/worldgen/biome/") {
            biome_26_2(&mut v, &mut report.lossy);
        }
        files.insert(path, serde_json::to_vec(&v).map_err(|e| e.to_string())?);
    }
    let (upgraded, up) = crate::upgrade::upgrade_pack(&files);
    for (what, n) in up.lossy {
        *report.lossy.entry(what).or_default() += n;
    }
    // A configured carver is a carver in 26.3: the old file gives way to the new one.
    for path in files.keys().filter(|p| !upgraded.contains_key(*p)) {
        let _ = fs::remove_file(inside.join(path));
    }
    for (path, bytes) in &upgraded {
        if !path.starts_with("data/") || path.split('/').any(|p| p == ".." || p.is_empty()) {
            continue;
        }
        let target = inside.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::write(&target, bytes).map_err(|e| format!("{}: {e}", target.display()))?;
        report.upgraded += 1;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_1_21_1_biome_has_its_colours_as_attributes_and_one_list_of_carvers() {
        let mut lossy = BTreeMap::new();
        let mut b = json!({"temperature": 0.8, "downfall": 0.0, "has_precipitation": false,
            "effects": {"sky_color": -16777216, "fog_color": 12638463, "water_color": 4159204,
                "water_fog_color": 329011},
            "carvers": {"air": ["minecraft:cave", "minecraft:canyon"], "liquid": "#x:wet"},
            "features": []});
        biome_26_2(&mut b, &mut lossy);
        assert_eq!(
            b["attributes"]["minecraft:visual/sky_color"],
            json!("#000000")
        );
        assert_eq!(
            b["attributes"]["minecraft:visual/fog_color"],
            json!("#c0d8ff")
        );
        assert_eq!(
            b["attributes"]["minecraft:visual/water_fog_color"],
            json!("#050533")
        );
        // The water colour stays an effect, as in 26.3.
        assert_eq!(b["effects"]["water_color"], json!(4159204));
        assert_eq!(b["carvers"], json!(["minecraft:cave", "minecraft:canyon"]));
        assert_eq!(lossy["biome: a tag of carvers"], 1);
        let mut empty = json!({"carvers": {}});
        biome_26_2(&mut empty, &mut lossy);
        assert_eq!(empty["carvers"], json!([]));
    }

    #[test]
    fn a_mods_dimension_is_written_in_26_3s_formats_with_names_of_its_own() {
        let dir = std::env::temp_dir().join(format!("hz-dimensions-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let write = |rel: &str, v: Value| {
            let p = dir.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, serde_json::to_vec(&v).unwrap()).unwrap();
        };
        let settings = |block: &str| {
            json!({"sea_level": 0, "disable_mob_generation": false, "aquifers_enabled": false,
                "ore_veins_enabled": false, "legacy_random_source": false,
                "default_block": {"Name": block}, "default_fluid": {"Name": "minecraft:water"},
                "noise": {"min_y": 0, "height": 128, "size_horizontal": 1, "size_vertical": 2},
                "noise_router": {"final_density": {"type": "minecraft:y_clamped_gradient",
                    "from_y": 0, "to_y": 128, "from_value": 1.0, "to_value": -1.0}},
                "spawn_target": [],
                "surface_rule": {"type": "minecraft:block", "result_state": {"Name": block}}})
        };
        for (name, block) in [("a", "m:wall"), ("b", "m:floor")] {
            write(
                &format!("data/m/dimension/{name}.json"),
                json!({"type": "m:rooms", "generator": {"type": "minecraft:noise",
                    "settings": settings(block),
                    "biome_source": {"type": "minecraft:fixed", "biome": "m:hall"}}}),
            );
        }
        write(
            "data/m/dimension_type/rooms.json",
            json!({"min_y": 0, "height": 128, "has_skylight": false}),
        );
        write(
            "data/m/worldgen/biome/hall.json",
            json!({"temperature": 0.5, "downfall": 0.5, "has_precipitation": false,
                "effects": {"sky_color": 0}, "spawners": {}, "spawn_costs": {}, "carvers": {},
                "features": []}),
        );
        let report = carry(&dir, &["m"]).unwrap();
        assert_eq!(report.dimensions, ["m:a", "m:b"]);
        assert!(report.upgraded >= 4);
        let read = |rel: &str| -> Value {
            serde_json::from_slice(&fs::read(dir.join(rel)).unwrap()).unwrap()
        };
        let a = read("data/m/dimension/a.json");
        let b = read("data/m/dimension/b.json");
        let (ra, rb) = (
            &a["generator"]["settings"]["material_rule"],
            &b["generator"]["settings"]["material_rule"],
        );
        // Each dimension's inline settings name a rule of its own.
        assert_eq!(ra, &json!("m:a/upgraded/material_rule"));
        assert_eq!(rb, &json!("m:b/upgraded/material_rule"));
        assert!(
            dir.join("data/m/worldgen/material_rule/a/upgraded/material_rule.json")
                .exists()
        );
        assert_eq!(a["generator"]["settings"]["default_block"], json!("m:wall"));
        let hall = read("data/m/worldgen/biome/hall.json");
        assert_eq!(hall["carvers"], json!([]));
        assert!(hall.get("spawners").is_none());
        // A second carry finds 26.3's formats and leaves them.
        let again = carry(&dir, &["m"]).unwrap();
        assert_eq!(again.upgraded, 0);
        let _ = fs::remove_dir_all(&dir);
    }
}
