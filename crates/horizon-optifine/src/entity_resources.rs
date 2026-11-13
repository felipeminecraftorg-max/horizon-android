//! Resource-pack overlay loader for partial native CEM and ETF compatibility.
//!
//! The caller supplies the same merged resource root as vanilla assets. No JARs or
//! scripts are executed, and invalid models/rules never replace a vanilla entity.
use crate::{entity_animations::Animation, entity_models, entity_textures, paths};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File}, io::{self, Read}, path::{Path, PathBuf},
};

const MAX_DOCUMENT: u64 = 1_048_576;
const MAX_FILES: usize = 4096;
const MAX_WARNINGS: usize = 1024;
const MAX_DEPTH: usize = 16;

/// Parsed model and compiled pose expressions. No GPU objects are allocated here.
#[derive(Clone, Debug)]
pub struct PreparedModel {
    pub model: entity_models::Model,
    pub animations: Vec<Animation>,
    pub unsupported_expressions: Vec<String>,
}

/// Models/rules indexed by namespaced resource path.
/// The original namespace and source resource path are kept to avoid collisions.
#[derive(Default, Debug)]
pub struct EntityResources {
    pub models: BTreeMap<String, PreparedModel>,
    /// Entity identifier to resolved active CEM resource path.
    pub models_by_entity: BTreeMap<String,String>,
    pub texture_rules: BTreeMap<String, entity_textures::TextureRules>,
    pub warnings: Vec<String>,
    /// Diagnostics omitted after reaching the mobile memory/telemetry cap.
    pub warnings_suppressed: usize,
    pub files_seen: usize,
}

fn read_bounded(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    if file.metadata()?.len() > MAX_DOCUMENT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "entity resource exceeds 1 MiB"));
    }
    let mut text = String::new();
    file.take(MAX_DOCUMENT + 1).read_to_string(&mut text)?;
    if text.len() as u64 > MAX_DOCUMENT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "entity resource exceeds 1 MiB"));
    }
    Ok(text)
}

/// Scan OptiFine CEM folders in enabled resource overlays, including old mcpatcher roots.
/// Folder links are followed (the overlay creates these); link loops are bounded.
fn walk_cem(
    dir: &Path, rel: &str, depth: usize,
    seen: &mut HashSet<PathBuf>, out: &mut Vec<(String, PathBuf)>,
) {
    if depth > MAX_DEPTH || out.len() >= MAX_FILES { return; }
    if !fs::canonicalize(dir).is_ok_and(|real| seen.insert(real)) { return; }
    let Ok(entries) = fs::read_dir(dir) else { return; };
    for entry in entries.flatten() {
        if out.len() >= MAX_FILES { break; }
        let filename = entry.file_name().to_string_lossy().into_owned();
        let child_rel = format!("{rel}/{filename}");
        let path = entry.path();
        match fs::metadata(&path) {
            Ok(info) if info.is_dir() => {
                walk_cem(&path, &child_rel, depth + 1, seen, out);
            }
            Ok(info) if info.is_file() && filename.ends_with(".jem") => {
                out.push((child_rel, path));
            }
            _ => {}
        }
    }
}

fn jem_files(root: &Path) -> Vec<(String,PathBuf)> {
    let Ok(namespaces) = fs::read_dir(root.join("assets")) else { return Vec::new(); };
    let mut files = Vec::new();
    let mut seen = HashSet::new();
    for ns in namespaces.flatten() {
        if files.len() >= MAX_FILES { break; }
        let name = ns.file_name().to_string_lossy().into_owned();
        for top in ["optifine", "mcpatcher"] {
            let rel = format!("assets/{name}/{top}/cem");
            walk_cem(&root.join(&rel), &rel, 0, &mut seen, &mut files);
        }
    }
    files.sort_by(|a,b| a.0.cmp(&b.0));
    files
}

fn collect_animations(
    part: &entity_models::Part,
    compiled: &mut Vec<Animation>,
    errors: &mut Vec<String>,
) {
    for (target, expr) in &part.animation {
        match Animation::compile(target, expr) {
            Ok(animation) => compiled.push(animation),
            Err(err) => errors.push(format!("{target}: {err:?}")),
        }
    }
    for child in &part.children { collect_animations(child, compiled, errors); }
}

impl PreparedModel {
    pub fn prepare(model: entity_models::Model) -> Self {
        let mut animations = Vec::new();
        let mut unsupported_expressions = Vec::new();
        for part in &model.parts {
            collect_animations(part, &mut animations, &mut unsupported_expressions);
        }
        Self { model, animations, unsupported_expressions }
    }
    /// Build model-space matrices after evaluating the supported animation channels.
    /// The result is data for a renderer, not a GPU upload or vanilla model override.
    pub fn pose<F: Fn(&str) -> Option<f64>>(
        &self, state: &F,
    ) -> Result<(crate::entity_pose::Pose, Vec<String>), crate::entity_pose::PoseError> {
        let mut pose = crate::entity_pose::Pose::from_model(&self.model)?;
        let (samples, mut warnings) = self.sample(state);
        warnings.extend(pose.apply_sample(&samples).into_iter()
            .map(|(key, err)| format!("{key}: {err:?}")));
        Ok((pose, warnings))
    }

    /// Sample supported CEM animation channels using the *current* entity render state.
    /// A renderer must map channel targets to actual model parts and restore vanilla
    /// pose values on failed expressions; the result is not applied to gameplay state.
    pub fn sample<F: Fn(&str) -> Option<f64>>(
        &self, state: &F,
    ) -> (Vec<(String,f32)>, Vec<String>) {
        let mut result = Vec::new();
        let mut failed = Vec::new();
        for binding in &self.animations {
            match binding.sample(state) {
                Ok(value) => result.push((binding.target.clone(), value)),
                Err(err) => failed.push(format!("{}: {err:?}", binding.target)),
            }
        }
        (result, failed)
    }
}

impl EntityResources {
    fn warn(&mut self, message: String) {
        if self.warnings.len() < MAX_WARNINGS {
            self.warnings.push(message);
        } else {
            self.warnings_suppressed += 1;
        }
    }

    /// Read CEM and ETF partial rules from the *merged* resource-pack root.
    /// Never enumerate an APK's private storage or arbitrary user-provided path.
    ///
    /// Notes: this loader does not yet install models or textures into the
    /// renderer's GPU resources. Unsupported features are recorded explicitly.
    pub fn load(root: &Path) -> Self {
        let mut resources = Self::default();
        for (relative, path) in jem_files(root) {
            if resources.files_seen >= MAX_FILES { break; }
            resources.files_seen += 1;
            let text = match read_bounded(&path) {
                Ok(text) => text, Err(err) => {
                    resources.warn(format!("{relative}: {err}")); continue;
                },
            };
            let dir = path.parent().unwrap_or(root);
            let dir_real = fs::canonicalize(dir).ok();
            let parsed = entity_models::parse_with(&text, |jpm| {
                // Both lexical validation (parser) and canonical containment
                // are required; overlay folder symlinks are legitimate.
                let real = fs::canonicalize(dir.join(jpm)).ok()?;
                if !dir_real.as_ref().is_some_and(|base| real.starts_with(base)) { return None; }
                read_bounded(&real).ok()
            });
            match parsed {
                Ok(model) => {
                    let prepared = PreparedModel::prepare(model);
                    for err in &prepared.unsupported_expressions {
                        resources.warn(format!("{relative}: {err}"));
                    }
                    if let Some(entity_id) = Self::cem_entity_id(&relative) {
                        resources.models_by_entity.insert(entity_id, relative.clone());
                    }
                    resources.models.insert(relative, prepared);
                },
                Err(err) => resources.warn(format!("{relative}: {err:?}")),
            }
        }
        for feature in ["random/entity", "mob"] {
            let files = paths::properties_files(root, feature);
            for relative in files.into_iter().take(MAX_FILES.saturating_sub(resources.files_seen)) {
                resources.files_seen += 1;
                match read_bounded(&root.join(&relative)) {
                    Ok(src) => match entity_textures::parse(&src) {
                        Ok(rules) => {
                            for skipped in &rules.skipped {
                                resources.warn(format!("{relative}: {skipped}"));
                            }
                            if !rules.rules.is_empty() {
                                resources.texture_rules.insert(relative, rules);
                            }
                        }
                        Err(err) => resources.warn(format!("{relative}: {err:?}")),
                    },
                    Err(err) => resources.warn(format!("{relative}: {err}")),
                }
            }
        }
        resources
    }

    /// Resolve an entity model for the vanilla render path without scanning files.
    /// Returns None for an unsupported/unloaded model; the caller should draw vanilla.
    pub fn model_for_entity(&self, entity_id: &str) -> Option<&PreparedModel> {
        self.models.get(self.models_by_entity.get(entity_id)?)
    }

    /// Resolve an ETF random texture rule set using the vanilla texture identifier.
    /// Example: minecraft:textures/entity/zombie/zombie.png.
    pub fn rules_for_texture(&self, texture_id: &str) -> Option<&entity_textures::TextureRules> {
        let (ns, rest) = texture_id.split_once(':')?;
        if !ns.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
            || rest.contains("..") || rest.contains('\\') { return None; }
        let path = rest.strip_prefix("textures/entity/")?.strip_suffix(".png")?;
        for top in ["optifine","mcpatcher"] {
            for feature in ["random/entity", "mob"] {
                let key = format!("assets/{ns}/{top}/{feature}/{path}.properties");
                if let Some(rules) = self.texture_rules.get(&key) { return Some(rules); }
            }
        }
        None
    }

    /// Namespaced entity ID associated with a source CEM path, e.g.
    /// assets/minecraft/optifine/cem/zombie.jem -> minecraft:zombie.
    pub fn cem_entity_id(path: &str) -> Option<String> {
        let segments: Vec<_> = path.split('/').collect();
        if segments.len() < 5 || segments[0] != "assets" ||
            !matches!(segments[2], "optifine" | "mcpatcher") ||
            segments[3] != "cem" { return None; }
        let namespace = segments[1];
        let name = segments.last()?.strip_suffix(".jem")?;
        if name.is_empty() { return None; }
        Some(format!("{namespace}:{name}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT: AtomicUsize = AtomicUsize::new(1);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let dir = std::env::temp_dir().join(
                format!("horizon-entity-resources-{}-{id}", std::process::id())
            );
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
        fn write(&self, path: &str, source: &str) {
            let p = self.0.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p,source).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }
    #[test]
    fn loads_jem_jpm_texture_rules_and_samples_pose() {
        let f = Fixture::new();
        f.write("assets/minecraft/optifine/cem/zombie.jem",
            r#"{"models":[{"part":"head","model":"head.jpm","animations":[{"head.rx":"sin(limb_swing)"}]}]}"#);
        f.write("assets/minecraft/optifine/cem/head.jpm",
            r#"{"boxes":[{"coordinates":[0,0,0,8,8,8]}]}"#);
        f.write("assets/minecraft/optifine/random/entity/zombie/zombie.properties",
            "skins.1=1 2\nweights.1=2 3");
        let loaded = EntityResources::load(&f.0);
        assert!(loaded.warnings.is_empty(), "{:?}",loaded.warnings);
        assert_eq!(loaded.models.len(),1);
        assert_eq!(loaded.texture_rules.len(),1);
        assert!(loaded.model_for_entity("minecraft:zombie").is_some());
        assert!(loaded.rules_for_texture("minecraft:textures/entity/zombie/zombie.png").is_some());
        let model = loaded.models.values().next().unwrap();
        assert_eq!(model.model.parts[0].boxes[0].coords[3],8.0);
        assert_eq!(model.sample(&|name| (name=="limb_swing").then_some(0.0)).0[0].1,0.0);
        let (pose,warnings) = model.pose(&|name| (name=="limb_swing").then_some(0.0)).unwrap();
        assert!(warnings.is_empty());
        assert!(pose.world_matrix("head").is_some());
    }
    #[test]
    fn broken_entries_do_not_cancel_valid_resources() {
        let f = Fixture::new();
        f.write("assets/minecraft/optifine/cem/zombie.jem", "{bad");
        f.write("assets/minecraft/optifine/cem/cow.jem",
            r#"{"models":[{"part":"head","animations":[{"head.rx":"unsupported(1)"}]}]}"#);
        let loaded = EntityResources::load(&f.0);
        assert_eq!(loaded.models.len(),1);
        assert!(loaded.warnings.len() >= 2);
        assert!(loaded.models.values().next().unwrap().animations.is_empty());
    }
    #[test]
    fn path_to_entity_id() {
        assert_eq!(EntityResources::cem_entity_id("assets/minecraft/optifine/cem/zombie.jem"),
            Some("minecraft:zombie".to_owned()));
        assert_eq!(EntityResources::cem_entity_id("assets/minecraft/models/zombie.jem"), None);
    }
}
