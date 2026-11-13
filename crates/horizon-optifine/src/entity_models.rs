//! Bounded parser for OptiFine CEM JEM/JPM definitions.
//! Rendering and posing are separate integrations: parsing never executes code.
use serde_json::Value;
use std::collections::HashSet;

const LIMIT: usize = 1_048_576;
const PART_LIMIT: usize = 512;
const BOX_LIMIT: usize = 4096;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    TooLarge, Json(String), Invalid(&'static str), Missing(String),
    InvalidPath(String), TooDeep, TooManyParts, TooManyBoxes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoxModel {
    pub coords: [f32; 6],
    pub uv: Option<[f32; 2]>,
    pub expand: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub name: String,
    pub id: String,
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub boxes: Vec<BoxModel>,
    pub children: Vec<Part>,
    pub animation: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub texture: Option<String>,
    pub texture_size: Option<[f32; 2]>,
    pub shadow: Option<f32>,
    pub parts: Vec<Part>,
}

fn vector<const N: usize>(v: &Value) -> Result<[f32; N], Error> {
    let a = v.as_array().ok_or(Error::Invalid("expected numeric array"))?;
    if a.len() != N { return Err(Error::Invalid("array size")); }
    let mut out = [0.0; N];
    for (i, value) in a.iter().enumerate() {
        let n = value.as_f64().ok_or(Error::Invalid("expected number"))?;
        if !n.is_finite() || n.abs() > 100_000.0 { return Err(Error::Invalid("number out of range")); }
        out[i] = n as f32;
    }
    Ok(out)
}
fn number(v: &Value) -> Result<f32, Error> {
    let n = v.as_f64().ok_or(Error::Invalid("expected number"))?;
    if !n.is_finite() || n.abs() > 100_000.0 { return Err(Error::Invalid("number out of range")); }
    Ok(n as f32)
}
fn safe_jpm(path: &str) -> bool {
    path.len() <= 256 && path.ends_with(".jpm") && !path.starts_with('/')
        && !path.contains('\\') && !path.contains(':')
        && path.split('/').all(|s| !s.is_empty() && s != "." && s != "..")
}
#[derive(Default)]
struct Count { parts: usize, boxes: usize }

fn part<F: FnMut(&str) -> Option<String>>(
    v: &Value, depth: usize, refs: &mut HashSet<String>, loader: &mut F, count: &mut Count,
) -> Result<Part, Error> {
    if depth > 12 { return Err(Error::TooDeep); }
    count.parts += 1;
    if count.parts > PART_LIMIT { return Err(Error::TooManyParts); }
    let obj = v.as_object().ok_or(Error::Invalid("part is not an object"))?;
    let external = if let Some(path) = obj.get("model").and_then(Value::as_str) {
        if !safe_jpm(path) || !refs.insert(path.to_owned()) {
            return Err(Error::InvalidPath(path.to_owned()));
        }
        let text = loader(path).ok_or_else(|| Error::Missing(path.to_owned()))?;
        if text.len() > LIMIT { return Err(Error::TooLarge); }
        let external = serde_json::from_str::<Value>(&text).map_err(|e| Error::Json(e.to_string()))?;
        refs.remove(path);
        Some(external)
    } else { None };
    let get = |key: &str| obj.get(key).or_else(|| external.as_ref().and_then(|e| e.get(key)));
    let name = get("part").and_then(Value::as_str).unwrap_or("").to_owned();
    let id = get("id").and_then(Value::as_str).unwrap_or(&name).to_owned();
    if name.len() > 128 || id.len() > 128 { return Err(Error::Invalid("part name too long")); }
    let translation = get("translate").map(vector).transpose()?.unwrap_or([0.0; 3]);
    let rotation = get("rotate").map(vector).transpose()?.unwrap_or([0.0; 3]);
    let boxes_json = get("boxes").and_then(Value::as_array).cloned().unwrap_or_default();
    count.boxes += boxes_json.len();
    if count.boxes > BOX_LIMIT { return Err(Error::TooManyBoxes); }
    let mut boxes = Vec::with_capacity(boxes_json.len());
    for b in &boxes_json {
        let coords = vector::<6>(b.get("coordinates").ok_or(Error::Invalid("box coordinates missing"))?)?;
        let uv = b.get("textureOffset").map(vector::<2>).transpose()?;
        let expand = b.get("sizeAdd").map(number).transpose()?.unwrap_or(0.0);
        boxes.push(BoxModel { coords, uv, expand });
    }
    let mut animation = Vec::new();
    if let Some(groups) = get("animations") {
        let groups = groups.as_array().ok_or(Error::Invalid("animations must be array"))?;
        for g in groups {
            let pairs = g.as_object().ok_or(Error::Invalid("animation must be object"))?;
            for (target, expression) in pairs {
                if animation.len() >= 256 || target.len() > 128 { return Err(Error::Invalid("animation count")); }
                let expression = expression.as_str().ok_or(Error::Invalid("expression must be string"))?;
                if expression.len() > 4096 { return Err(Error::Invalid("expression too long")); }
                animation.push((target.clone(), expression.to_owned()));
            }
        }
    }
    let mut children = Vec::new();
    if let Some(items) = get("submodels") {
        let items = items.as_array().ok_or(Error::Invalid("submodels must be array"))?;
        for child in items { children.push(part(child, depth + 1, refs, loader, count)?); }
    }
    Ok(Part { name, id, translation, rotation, boxes, children, animation })
}

/// Resolve JPM references only within the owning resource pack directory.
/// The callback must never use untrusted paths without joining to a fixed root.
pub fn parse_with<F: FnMut(&str) -> Option<String>>(src: &str, mut loader: F) -> Result<Model, Error> {
    if src.len() > LIMIT { return Err(Error::TooLarge); }
    let root: Value = serde_json::from_str(src).map_err(|e| Error::Json(e.to_string()))?;
    let models = root.get("models").and_then(Value::as_array).ok_or(Error::Invalid("models array missing"))?;
    let texture = root.get("texture").map(|v| v.as_str().map(str::to_owned)
        .ok_or(Error::Invalid("texture must be string"))).transpose()?;
    let texture_size = root.get("textureSize").map(vector).transpose()?;
    let shadow = root.get("shadowSize").map(number).transpose()?;
    let mut count = Count::default();
    let mut refs = HashSet::new();
    let mut parts = Vec::new();
    for v in models { parts.push(part(v, 0, &mut refs, &mut loader, &mut count)?); }
    Ok(Model { texture, texture_size, shadow, parts })
}
pub fn parse(src: &str) -> Result<Model, Error> { parse_with(src, |_| None) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn embedded_model() {
        let m = parse(r#"{"textureSize":[64,32],"models":[{"part":"head","boxes":[{"coordinates":[-4,-8,-4,8,8,8],"textureOffset":[0,0]}],"animations":[{"head.rx":"sin(limb_swing)"}]}]}"#).unwrap();
        assert_eq!(m.parts[0].boxes.len(), 1);
        assert_eq!(m.parts[0].animation[0].0, "head.rx");
    }
    #[test] fn external_model() {
        let m = parse_with(r#"{"models":[{"part":"head","model":"head.jpm"}]}"#, |_| Some(
            r#"{"boxes":[{"coordinates":[0,0,0,4,4,4]}]}"#.to_owned())).unwrap();
        assert_eq!(m.parts[0].boxes[0].coords[3], 4.0);
    }
    #[test] fn blocks_traversal() {
        assert!(matches!(parse_with(r#"{"models":[{"model":"../x.jpm"}]}"#, |_| None),
            Err(Error::InvalidPath(_))));
    }
    #[test] fn rejects_bad_geometry() {
        assert!(parse(r#"{"models":[{"boxes":[{"coordinates":[0,1,2]}]}]}"#).is_err());
    }
    #[test] fn rejects_depth_bomb() {
        let mut v = serde_json::json!({"part":"head"});
        for _ in 0..15 { v = serde_json::json!({"submodels":[v]}); }
        let m = serde_json::json!({"models":[v]});
        assert_eq!(parse(&m.to_string()), Err(Error::TooDeep));
    }
}
