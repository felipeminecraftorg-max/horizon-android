//! The levels a world has besides the Overworld, the Nether and the End: the dimensions the mods
//! and the world's data packs define (`data/<namespace>/dimension/<path>.json`, a level stem, and
//! the dimension type it names), as `WorldDimensions` keeps them.
//!
//! The world that opens registers them ([`set`]), the server thread and the window alike; a level,
//! the chunk layout and the network find one by its id, the game by its place in the list.
use crate::{chunk::LevelShape, datapack::DataPack};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, RwLock},
};

/// The vanilla levels, which are never data dimensions here.
pub const VANILLA: [&str; 3] = [
    "minecraft:overworld",
    "minecraft:the_nether",
    "minecraft:the_end",
];

/// What a level's sky looks like: its dimension type's `effects` (1.21.1) or `skybox` and
/// `cardinal_light` (26.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sky {
    Overworld,
    Nether,
    End,
}

/// A dimension of a mod's or a data pack's.
#[derive(Clone, Debug)]
pub struct DataDimension {
    /// `namespace:path`, the level's key.
    pub id: &'static str,
    /// The dimension type the stem names (its id; the stem's own id when the type is inline).
    pub kind: String,
    pub shape: LevelShape,
    pub has_ceiling: bool,
    /// `ambient_light`: how bright the darkest light is.
    pub ambient_light: f32,
    /// `fixed_time`: the time of day it always is.
    pub fixed_time: Option<i64>,
    /// `coordinate_scale`: travel between levels scales x and z by the ratio of theirs.
    pub coordinate_scale: f64,
    /// `ultrawarm`: water evaporates, lava flows fast.
    pub ultrawarm: bool,
    pub sky: Sky,
    /// The dimension type as its pack wrote it.
    pub type_json: Arc<Value>,
}

impl DataDimension {
    /// A level stem `id` and the dimension type it names (`types` by id: the packs').
    /// `None` when the type is missing or its build range is one the codec refuses.
    pub fn from_stem(id: &str, stem: &Value, types: &BTreeMap<String, Value>) -> Option<Self> {
        let (kind, ty) = match &stem["type"] {
            Value::String(t) => (
                t.clone(),
                types.get(t).cloned().or_else(|| vanilla_type(t))?,
            ),
            inline @ Value::Object(_) => (id.to_owned(), inline.clone()),
            _ => return None,
        };
        let int = |k: &str| ty[k].as_i64().and_then(|v| i32::try_from(v).ok());
        let (min_y, height) = (int("min_y")?, int("height")?);
        let logical_height = int("logical_height").unwrap_or(height);
        let sky_light = ty["has_skylight"].as_bool().unwrap_or(true);
        let shape = LevelShape::new(min_y, height, logical_height, sky_light).ok()?;
        // 1.21.1's field, or 26.3's attribute.
        let ultrawarm = ty["ultrawarm"]
            .as_bool()
            .or_else(|| ty["attributes"]["minecraft:gameplay/water_evaporates"].as_bool())
            .unwrap_or(false);
        let effects = ty["effects"].as_str().unwrap_or_default();
        let sky = match (
            effects.trim_start_matches("minecraft:"),
            ty["skybox"].as_str(),
            ty["cardinal_light"].as_str(),
        ) {
            ("the_nether", ..) | (_, _, Some("nether")) => Sky::Nether,
            ("the_end", ..) | (_, Some("end"), _) => Sky::End,
            _ => Sky::Overworld,
        };
        let has_ceiling = ty["has_ceiling"].as_bool().unwrap_or(false);
        let ambient_light = ty["ambient_light"].as_f64().unwrap_or(0.0) as f32;
        let fixed_time = ty["fixed_time"].as_i64();
        let coordinate_scale = ty["coordinate_scale"]
            .as_f64()
            .filter(|s| *s > 0.0)
            .unwrap_or(1.0);
        Some(Self {
            id: intern(id),
            kind,
            shape,
            has_ceiling,
            ambient_light,
            fixed_time,
            coordinate_scale,
            ultrawarm,
            sky,
            type_json: Arc::new(ty),
        })
    }
}

/// A vanilla dimension type a pack's level may name, as far as these levels read it.
fn vanilla_type(id: &str) -> Option<Value> {
    use serde_json::json;
    Some(match id {
        "minecraft:overworld" | "minecraft:overworld_caves" => json!({"min_y": -64, "height": 384,
            "logical_height": 384, "has_skylight": true, "has_ceiling": id.ends_with("caves"),
            "effects": "minecraft:overworld"}),
        "minecraft:the_nether" => json!({"min_y": 0, "height": 256, "logical_height": 128,
            "has_skylight": false, "has_ceiling": true, "ambient_light": 0.1, "ultrawarm": true,
            "coordinate_scale": 8.0, "fixed_time": 18000, "effects": "minecraft:the_nether"}),
        "minecraft:the_end" => json!({"min_y": 0, "height": 256, "logical_height": 256,
            "has_skylight": true, "ambient_light": 0.25, "fixed_time": 6000,
            "effects": "minecraft:the_end"}),
        _ => return None,
    })
}

/// The data dimensions `packs` define (lowest priority first: a later pack's file for an id
/// replaces an earlier one's), in id order. The vanilla levels' stems are left to the generator.
pub fn load(packs: &[&DataPack]) -> Vec<DataDimension> {
    let stems = crate::datapack::dimension_stems(packs);
    let mut types = BTreeMap::new();
    for pack in packs {
        for file in pack.list_kinds(&["dimension_type"]) {
            let Some((namespace, path)) = file
                .strip_prefix("data/")
                .and_then(|r| r.split_once("/dimension_type/"))
                .and_then(|(ns, p)| Some((ns, p.strip_suffix(".json")?)))
            else {
                continue;
            };
            if let Some(json) = pack
                .read(&file)
                .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            {
                types.insert(format!("{namespace}:{path}"), json);
            }
        }
    }
    stems
        .iter()
        .filter(|(id, _)| !VANILLA.contains(&id.as_str()))
        .filter_map(|(id, stem)| match DataDimension::from_stem(id, stem, &types) {
            Some(d) => Some(d),
            None => {
                log::warn!(
                    "Horizon: dimension {id}: its dimension type is missing or has a build height \
                     the game refuses; the level is left out"
                );
                None
            }
        })
        .collect()
}

/// The world's data dimensions, the list the open world registered.
static LIST: RwLock<Option<Arc<Vec<DataDimension>>>> = RwLock::new(None);

/// Every id a list has held, kept for the whole run: a level's id is `'static`.
static NAMES: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());

fn intern(id: &str) -> &'static str {
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(name) = names.iter().find(|n| **n == id) {
        return name;
    }
    let name: &'static str = Box::leak(id.to_owned().into_boxed_str());
    names.push(name);
    name
}

/// The open world's data dimensions: called as it opens, before any level is looked up.
pub fn set(list: Vec<DataDimension>) {
    if !list.is_empty() {
        let ids: Vec<&str> = list.iter().map(|d| d.id).collect();
        log::info!("Horizon: the world's own dimensions: {}", ids.join(", "));
    }
    *LIST.write().unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(list));
}

/// The registered data dimensions, in id order.
pub fn all() -> Arc<Vec<DataDimension>> {
    LIST.read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default()
}

/// The place of `id` in [`all`].
pub fn index(id: &str) -> Option<u16> {
    all()
        .iter()
        .position(|d| d.id == id)
        .and_then(|i| u16::try_from(i).ok())
}

/// The data dimension at `index` in [`all`].
pub fn get(index: u16) -> Option<DataDimension> {
    all().get(usize::from(index)).cloned()
}

/// The id of the data dimension at `index` in [`all`].
pub fn id_at(index: u16) -> Option<&'static str> {
    all().get(usize::from(index)).map(|d| d.id)
}

/// The data dimension `id`.
pub fn by_id(id: &str) -> Option<DataDimension> {
    all().iter().find(|d| d.id == id).cloned()
}

/// The build range of the data dimension `id`.
pub fn shape(id: &str) -> Option<LevelShape> {
    all().iter().find(|d| d.id == id).map(|d| d.shape)
}

/// The dimension type of the level `id`: a data dimension's own, a vanilla level's name.
pub fn type_of(id: &str) -> String {
    all()
        .iter()
        .find(|d| d.id == id)
        .map_or_else(|| id.to_owned(), |d| d.kind.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_stem_takes_its_shape_and_sky_from_the_type_it_names() {
        let types = BTreeMap::from([(
            "pack:rooms".to_owned(),
            json!({"min_y": 0, "height": 128, "logical_height": 128, "has_skylight": false,
                "has_ceiling": true, "ambient_light": 0.0, "fixed_time": 16000,
                "effects": "pack:rooms"}),
        )]);
        let d = DataDimension::from_stem(
            "pack:level",
            &json!({"type": "pack:rooms", "generator": {}}),
            &types,
        )
        .unwrap();
        assert_eq!(d.id, "pack:level");
        assert_eq!(d.kind, "pack:rooms");
        assert_eq!((d.shape.min_y, d.shape.height), (0, 128));
        assert!(!d.shape.sky_light && d.has_ceiling);
        assert_eq!(d.fixed_time, Some(16000));
        assert_eq!(d.sky, Sky::Overworld);
        // The same id gives the same 'static string.
        assert!(std::ptr::eq(d.id, intern("pack:level")));
    }

    #[test]
    fn a_type_the_codec_refuses_or_a_missing_one_leaves_the_level_out() {
        let types = BTreeMap::from([("pack:odd".to_owned(), json!({"min_y": 3, "height": 128}))]);
        assert!(DataDimension::from_stem("pack:a", &json!({"type": "pack:odd"}), &types).is_none());
        assert!(
            DataDimension::from_stem("pack:a", &json!({"type": "pack:none"}), &types).is_none()
        );
        let inline = json!({"type": {"min_y": -64, "height": 384, "skybox": "end"}});
        let d = DataDimension::from_stem("pack:b", &inline, &types).unwrap();
        assert_eq!((d.kind.as_str(), d.sky), ("pack:b", Sky::End));
    }
}
