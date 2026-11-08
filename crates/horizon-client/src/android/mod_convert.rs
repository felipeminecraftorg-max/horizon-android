//! The mod and modpack converter: a Java mod's `.jar`, a modpack file (`.mrpack`, a CurseForge
//! zip) or a Modrinth modpack link becomes folder mods in `horizon-mods/`, turned on beside the
//! mods already there (Horizon's own included), with `horizon-port-pack`.
//!
//! What a jar carries (blocks, items, recipes, loot, tags, models, textures, sounds, lang,
//! structures and the rest of its worldgen) is converted to 26.3. What only its Java code does
//! (mobs' AI, machines, renderers, Mixins, event handlers) does not run until Horizon has a port
//! of it. A pack is carried over as a whole, so mods that share a library's namespace (YUNG's)
//! and worldgen that names another mod of the pack keep working together.
//!
//! The converter is hybrid, in lanes: (1) data, carried over by `horizon-port-pack`; (4) known
//! mods, whose Java code Horizon has its own version of (`KNOWN`, `game/android_bundled/compat.rs`),
//! turned on for a pack that has them; and what is left, which the prompt sorts by the lane that
//! would cover it: (2) behaviour as data (entities, worldgen types, block shapes), (3) behaviour
//! callbacks, (4) native ports for engine changes.
//!
//! Nothing is dropped without a word. Every file of a pack, and every left-out part of each mod,
//! ends in one file beside the mods, `horizon-mods/<name>-para-o-Claude.txt` (Files > Mods):
//! the report for the player, then a prompt that asks for the ports still missing, with each
//! mod's jar (name, SHA-1, link) and what its code does.
use crate::browser;
use horizon_port_pack::{
    check::{self, Check, Verdict},
    fetch::{self, How},
    import, inventory, pipeline,
    report::{Row, Status as CoverageStatus},
    resourcepacks,
};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

/// Where a conversion reports how far it is: a line and a share done (0..1).
pub type Progress<'a> = &'a dyn Fn(&str, f32);

/// Where packs are converted (their work folders) under the game folder.
const WORK: &str = "modpacks";

/// Java mods whose code Horizon runs in a version of its own (`game/android_bundled/compat.rs`):
/// the mod ids and Modrinth slugs that ask for it, the built-in mod it turns on, its title, and
/// how Horizon's version plays.
static KNOWN: [(&[&str], &str, &str, &str); 6] = [
    (
        &["rightclickharvest", "right-click-harvest"],
        "rightclickharvest",
        "Right Click Harvest",
        "right-click a grown crop (wheat, carrots, potatoes, beetroots, nether wart, cocoa, a mod's \
         crops) to harvest it and replant it",
    ),
    (
        &[
            "treeharvester",
            "tree-harvester",
            "fallingtree",
            "falling-tree",
        ],
        "treeharvester",
        "Tree Harvester",
        "breaking a log of a natural tree with an axe fells the tree, up to 256 logs; sneak to \
         break one log",
    ),
    (
        &[
            "veinminer",
            "vein-miner",
            "veinmining",
            "vein-mining",
            "ftbultimine",
            "ftb-ultimine",
        ],
        "veinminer",
        "Veinminer",
        "sneak while mining an ore with the right tool to mine its vein, up to 64 blocks (the \
         Java mod's key is sneak here); it stops before the tool breaks",
    ),
    (
        &[
            "sound_physics_remastered",
            "sound-physics-remastered",
            "soundphysics",
            "sound-physics",
        ],
        "sound_physics_remastered",
        "Sound Physics",
        "sounds are muffled by the blocks between you and them, echo in rooms and caves (longer \
         in bigger ones) and are dulled while your head is under water",
    ),
    (
        &[
            "firstperson",
            "first-person-model",
            "firstpersonmod",
            "first-person",
        ],
        "firstperson",
        "First-person Model",
        "in first person, look down to see your body and legs, with your own arms holding \
         your items (or the usual first-person hands: its settings in Options > Mods)",
    ),
    (
        &["cameraoverhaul", "camera-overhaul", "camera_overhaul"],
        "cameraoverhaul",
        "CameraOverhaul",
        "the camera rolls as you strafe and turn, tilts with your speed and falls, sways gently \
         while you stand still and shakes from explosions, thunder and swings (settings in \
         Options > Mods)",
    ),
];

/// The known mod one of `names` (a mod id, a slug, a name) is.
fn known(
    names: &[&str],
) -> Option<&'static (
    &'static [&'static str],
    &'static str,
    &'static str,
    &'static str,
)> {
    KNOWN.iter().find(|(ids, ..)| {
        names
            .iter()
            .any(|n| !n.is_empty() && (ids.contains(n) || ids.contains(&slug(n).as_str())))
    })
}

/// A known mod one of `names` asks for (the Mods page's Install): Horizon's own version turned
/// on in its place, and the launcher's line. None for any other mod.
pub fn turn_on_known(game: &Path, names: &[&str]) -> Option<String> {
    let (_, id, title, how) = known(names)?;
    Some(match browser::java::set_enabled(game, id, true) {
        Ok(()) => format!(
            "{title} needs no download: Horizon runs its own version, now on (Installed > \
             {title}). In game, {how}."
        ),
        Err(e) => format!("{title}: Horizon's own version could not be turned on: {e}"),
    })
}

/// A known mod's entry: Horizon's version turned on, what it does in place of the Java code.
fn cover(entry: &mut Entry, game: &Path, names: &[&str]) {
    let Some((_, id, title, how)) = known(names) else {
        return;
    };
    match browser::java::set_enabled(game, id, true) {
        Ok(()) => {
            entry.status = Status::Works;
            entry.reason =
                format!("Horizon runs its own version of it, now on (Installed > {title}): {how}");
            entry.left.clear();
            entry.features.clear();
        }
        Err(e) => {
            entry.reason = format!("{}; Horizon's own version not turned on: {e}", entry.reason)
        }
    }
}

/// The prepared vanilla version the game plays, when it is there: block templates are limited
/// to its blocks.
pub fn resources() -> Option<PathBuf> {
    let dir = crate::backend::data_dir()
        .join(".local/horizon-client")
        .join(horizon_core::VANILLA_VERSION);
    dir.is_dir().then_some(dir)
}

/// The modpack a Mods search names: `https://modrinth.com/modpack/<slug>` (any of Modrinth's
/// modpack links) or `modpack:<slug>`.
pub fn modpack_slug(query: &str) -> Option<String> {
    let q = query.trim();
    let rest = q
        .strip_prefix("modpack:")
        .or_else(|| q.split_once("modrinth.com/modpack/").map(|(_, rest)| rest))?;
    let slug: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect();
    (!slug.is_empty()).then_some(slug)
}

/// A pack's or mod's file name: its name in lowercase words joined by dashes.
fn slug(name: &str) -> String {
    let words: Vec<String> = name
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    if words.is_empty() {
        "mod".into()
    } else {
        words.join("-")
    }
}

fn copy_dir(from: &Path, to: &Path) -> Result<(), String> {
    fs::create_dir_all(to).map_err(|e| format!("{}: {e}", to.display()))?;
    for entry in fs::read_dir(from).map_err(|e| format!("{}: {e}", from.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            copy_dir(&src, &dst)?;
        } else if kind.is_file() {
            fs::copy(&src, &dst).map_err(|e| format!("{}: {e}", dst.display()))?;
        }
    }
    Ok(())
}

/// What became of one file.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    /// Converted and on; everything it carries runs.
    Works,
    /// Converted and on; what only its Java code does is listed as left out.
    Partly,
    /// Nothing of it runs yet: everything it does is Java code.
    NotYet,
    /// Converted, but not turned on (it would clash with a mod already on).
    Off,
    /// Its jar could not be downloaded.
    NotDownloaded,
    /// Horizon has its own (a library, a performance mod, a client tweak).
    NotNeeded,
}

impl Status {
    const ALL: [Status; 6] = [
        Status::Works,
        Status::Partly,
        Status::NotYet,
        Status::Off,
        Status::NotDownloaded,
        Status::NotNeeded,
    ];
    fn heading(self) -> &'static str {
        match self {
            Status::Works => "FUNCIONA (convertido e ligado)",
            Status::Partly => "PARCIAL (ligado; falta o que é código Java, listado)",
            Status::NotYet => "AINDA NÃO (tudo o que ele faz é código Java)",
            Status::Off => "CONVERTIDO MAS NÃO LIGADO",
            Status::NotDownloaded => "NÃO BAIXADO",
            Status::NotNeeded => "NÃO PRECISA (o Horizon tem o seu)",
        }
    }
    /// Whether the prompt asks for work on it.
    fn missing(self) -> bool {
        matches!(
            self,
            Status::Partly | Status::NotYet | Status::Off | Status::NotDownloaded
        )
    }
}

/// One file of a conversion, for the report and the prompt.
struct Entry {
    file: String,
    name: String,
    version: String,
    mod_id: String,
    loader: String,
    license: String,
    status: Status,
    reason: String,
    ports: Vec<String>,
    left: Vec<String>,
    /// Classes with behaviour and the methods they override, for the prompt.
    examples: Vec<String>,
    events: Vec<String>,
    /// What its code does, for the lanes the prompt names.
    features: Vec<horizon_port_pack::classes::Feature>,
    /// What lane 2 (its behaviour as data) made of it.
    lane2: Option<String>,
    /// What lane 3 (its Java code, run by Android's runtime) made of it.
    lane3: Option<String>,
    /// The class or member of Minecraft's or Fabric's API its code stopped at.
    missing_api: Option<String>,
    sha1: Option<String>,
    links: Vec<String>,
}

impl Entry {
    fn plain(file: &str, status: Status, reason: impl Into<String>) -> Self {
        Entry {
            file: file.to_owned(),
            name: file.to_owned(),
            version: String::new(),
            mod_id: String::new(),
            loader: String::new(),
            license: String::new(),
            status,
            reason: reason.into(),
            ports: Vec::new(),
            left: Vec::new(),
            examples: Vec::new(),
            events: Vec::new(),
            features: Vec::new(),
            lane2: None,
            lane3: None,
            missing_api: None,
            sha1: None,
            links: Vec::new(),
        }
    }

    fn checked(file: &str, check: &Check, status: Status, reason: String) -> Self {
        let mut e = Entry::plain(file, status, reason);
        e.name = check.name.clone();
        e.version = check.version.clone();
        e.mod_id = check.mod_id.clone();
        e.loader = check.loader.clone();
        e.license = check.license.clone();
        e.ports = check.ports.clone();
        e.left = check.skipped.clone();
        e.examples = check
            .code
            .examples
            .iter()
            .take(12)
            .map(|(class, methods)| {
                if methods.is_empty() {
                    class.clone()
                } else {
                    format!("{class}: {}", methods.join(", "))
                }
            })
            .collect();
        e.events = check.code.events.iter().cloned().collect();
        e.features = check.code.features.keys().copied().collect();
        e
    }
}

/// The mods in `horizon-mods/`: who owns which namespace, so a converted mod never takes one
/// from a mod already on (the game would refuse the later of the two).
struct Mods<'a> {
    game: &'a Path,
    dir: PathBuf,
    /// Namespace (a mod's id among them) → the id of the mod that owns it.
    owners: BTreeMap<String, String>,
    /// The ids of Horizon packages (`.hmod`), which a conversion never replaces.
    packaged: BTreeSet<String>,
}

impl<'a> Mods<'a> {
    fn load(game: &'a Path) -> Self {
        let dir = game.join(browser::java::MODS_DIR);
        let mut owners = BTreeMap::new();
        let mut packaged = BTreeSet::new();
        for path in horizon_mods::manifest::find(&dir) {
            let manifest = if path.is_dir() {
                fs::read_to_string(path.join(horizon_mods::manifest::MANIFEST))
                    .map_err(|e| e.to_string())
                    .and_then(|t| horizon_mods::Manifest::parse(&t))
            } else {
                horizon_mods::Package::read(&path).map(|p| p.manifest)
            };
            if let Ok(m) = manifest {
                if !path.is_dir() {
                    packaged.insert(m.id.clone());
                }
                for n in std::iter::once(&m.id).chain(&m.namespaces) {
                    owners.entry(n.clone()).or_insert_with(|| m.id.clone());
                }
            }
        }
        Mods {
            game,
            dir,
            owners,
            packaged,
        }
    }

    /// Copies the carried-over folder mod `carry` into `horizon-mods/<id>/` (replacing an older
    /// conversion of the same mod) and turns it on. Returns its id.
    fn install(&mut self, carry: &Path) -> Result<String, String> {
        let manifest = fs::read_to_string(carry.join(horizon_mods::manifest::MANIFEST))
            .map_err(|e| format!("no manifest: {e}"))
            .and_then(|t| horizon_mods::Manifest::parse(&t))?;
        let id = manifest.id.clone();
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(format!("its id {id:?} is not a folder name"));
        }
        if horizon_client::mod_list::ANDROID_BUNDLED
            .iter()
            .any(|(bundled, ..)| *bundled == id)
        {
            return Err(format!("Horizon has a built-in mod with the id {id}"));
        }
        if self.packaged.contains(&id) {
            return Err(format!("the Horizon mod {id}.hmod is already installed"));
        }
        let names: Vec<&String> = std::iter::once(&id).chain(&manifest.namespaces).collect();
        if let Some((n, owner)) = names
            .iter()
            .find_map(|n| self.owners.get(*n).filter(|o| **o != id).map(|o| (n, o)))
        {
            return Err(format!(
                "it would share the namespace {n} with {owner}, which is already on"
            ));
        }
        fs::create_dir_all(&self.dir).map_err(|e| format!("{}: {e}", self.dir.display()))?;
        let part = self.dir.join(format!(".{id}.part"));
        let _ = fs::remove_dir_all(&part);
        copy_dir(carry, &part).inspect_err(|_| {
            let _ = fs::remove_dir_all(&part);
        })?;
        let target = self.dir.join(&id);
        let _ = fs::remove_dir_all(&target);
        fs::rename(&part, &target).map_err(|e| format!("{}: {e}", target.display()))?;
        browser::java::set_enabled(self.game, &id, true)?;
        for n in names {
            self.owners.insert(n.clone(), id.clone());
        }
        Ok(id)
    }
}

/// The status a checked, carried-over mod gets once its install was tried.
fn installed(file: &str, check: &Check, install: Result<String, String>) -> Entry {
    match install {
        Ok(_) if check.verdict == Verdict::Works => {
            Entry::checked(file, check, Status::Works, check.summary.clone())
        }
        Ok(_) => Entry::checked(file, check, Status::Partly, check.summary.clone()),
        Err(why) => Entry::checked(file, check, Status::Off, why),
    }
}

/// Gives a carried-over mod its creative inventory tab, which a Java mod makes in code the carry
/// can't take: every item it has (its client item definitions), titled as its own tab was (the
/// language file's `item_group` name) or by the mod's name, in the order its language file
/// names them. A mod with a tab of its own keeps that.
fn creative_tab(carry: &Path, id: &str, name: &str) {
    let dir = carry.join("data").join(id).join("horizon/creative_tab");
    if dir.is_dir() {
        return;
    }
    let mut items = Vec::new();
    for ns in fs::read_dir(carry.join("assets"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let namespace = ns.file_name().to_string_lossy().into_owned();
        if namespace == "minecraft" {
            continue;
        }
        for file in fs::read_dir(ns.path().join("items"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let file_name = file.file_name().to_string_lossy().into_owned();
            if let Some(stem) = file_name.strip_suffix(".json") {
                items.push((namespace.clone(), stem.to_owned()));
            }
        }
    }
    if items.is_empty() {
        return;
    }
    let lang = fs::read_to_string(carry.join("assets").join(id).join("lang/en_us.json"))
        .unwrap_or_default();
    let at = |ns: &str, path: &str| {
        ["item", "block"]
            .iter()
            .filter_map(|kind| lang.find(&format!("\"{kind}.{ns}.{path}\"")))
            .min()
            .unwrap_or(usize::MAX)
    };
    let mut ordered: Vec<(usize, String)> = items
        .iter()
        .map(|(ns, path)| (at(ns, path), format!("{ns}:{path}")))
        .collect();
    ordered.sort();
    let items: Vec<String> = ordered.into_iter().map(|(_, item)| item).collect();
    let titles: BTreeMap<String, String> = serde_json::from_str(&lang).unwrap_or_default();
    let title = titles
        .iter()
        .filter(|(k, _)| k.starts_with("item_group.") || k.starts_with("itemGroup."))
        .min_by_key(|(k, _)| lang.find(&format!("\"{k}\"")).unwrap_or(usize::MAX))
        .map(|(_, v)| v.clone())
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| {
            if name.is_empty() {
                id.to_owned()
            } else {
                name.to_owned()
            }
        });
    let json = serde_json::json!({"title": title, "icon": items[0], "items": items});
    if fs::create_dir_all(&dir).is_ok() {
        let _ = fs::write(dir.join(format!("{id}.json")), json.to_string());
    }
}

/// Lane 2, block shapes: the carried blocks whose classes make their own shape take their
/// models' (`horizon_port_pack::shapes`). Returns how many did.
fn model_shapes(carry: &Path, check: &Check) -> usize {
    horizon_port_pack::shapes::add_model_shapes(carry, &check.mod_id, &check.code.shaped)
}

fn with_shapes(mut entry: Entry, shaped: usize) -> Entry {
    if shaped > 0 {
        let line = format!("{shaped} blocks take their shape from their models");
        entry.lane2 = Some(match entry.lane2.take() {
            Some(other) => format!("{line}; {other}"),
            None => line,
        });
    }
    entry
}

/// A PNG's width and height, from its header.
fn png_size(path: &Path) -> Option<(u32, u32)> {
    use std::io::Read;
    let mut head = [0u8; 24];
    fs::File::open(path).ok()?.read_exact(&mut head).ok()?;
    if &head[..8] != b"\x89PNG\r\n\x1a\n" || &head[12..16] != b"IHDR" {
        return None;
    }
    let n = |at: usize| u32::from_be_bytes([head[at], head[at + 1], head[at + 2], head[at + 3]]);
    Some((n(16), n(20)))
}

fn pngs(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "png"))
        .collect();
    out.sort();
    out
}

/// Lane 2, mob looks: the textures a carried-over mod gives vanilla mobs (`assets/<ns>/textures/
/// entity/<vanilla folder>/<name>.png` the size of that folder's vanilla textures, so made for
/// the vanilla model), as `(folder, texture)`. Layers (eyes, overlays, saddles) and babies'
/// textures stay out.
fn mob_looks(carry: &Path, resources: Option<&Path>) -> Vec<(String, String)> {
    const LAYERS: [&str; 10] = [
        "eyes", "overlay", "glow", "emissive", "saddle", "collar", "armor", "baby", "layer",
        "outer",
    ];
    let Some(resources) = resources else {
        return Vec::new();
    };
    let vanilla = resources.join("assets/minecraft/textures/entity");
    let mut out = Vec::new();
    for ns in fs::read_dir(carry.join("assets"))
        .into_iter()
        .flatten()
        .flatten()
    {
        let namespace = ns.file_name().to_string_lossy().into_owned();
        if namespace == "minecraft" {
            continue;
        }
        let entity = ns.path().join("textures/entity");
        for dir in fs::read_dir(&entity).into_iter().flatten().flatten() {
            let folder = dir.file_name().to_string_lossy().into_owned();
            if folder == "player" || !dir.path().is_dir() {
                continue;
            }
            let sizes: BTreeSet<(u32, u32)> = pngs(&vanilla.join(&folder))
                .iter()
                .filter_map(|p| png_size(p))
                .collect();
            if sizes.is_empty() {
                continue;
            }
            for png in pngs(&dir.path()) {
                let stem = png
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_ascii_lowercase())
                    .unwrap_or_default();
                if LAYERS.iter().any(|l| stem.contains(l))
                    || !png_size(&png).is_some_and(|s| sizes.contains(&s))
                {
                    continue;
                }
                out.push((
                    folder.clone(),
                    format!("{namespace}:entity/{folder}/{stem}"),
                ));
            }
        }
    }
    out
}

/// Keeps mod `id`'s mob looks in the game's table (`mobs/variants.rs`), in place of its old ones.
fn record_looks(game: &Path, id: &str, looks: &[(String, String)]) -> Result<(), String> {
    let path = game.join(horizon_client::mobs::variants::FILE);
    let old = fs::read_to_string(&path).unwrap_or_default();
    let prefix = format!("{id}\t");
    let mut lines: Vec<String> = old
        .lines()
        .filter(|l| !l.starts_with(&prefix))
        .map(str::to_owned)
        .collect();
    lines.extend(
        looks
            .iter()
            .map(|(folder, texture)| format!("{id}\t{folder}\t{texture}")),
    );
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    fs::write(&path, lines.join("\n") + "\n").map_err(|e| e.to_string())
}

/// Lane 2 for a carried-over mod with no entities of its own: its looks for vanilla mobs. A mod
/// with nothing else to run (`Doesn't run`) is installed for them. Returns the entry as it
/// comes out.
fn lane2(game: &Path, carry: &Path, check: &Check, mods: &mut Mods, entry: Entry) -> Entry {
    use horizon_port_pack::classes::Feature as F;
    if check.code.features.contains_key(&F::Entity)
        || check.code.features.contains_key(&F::Renderer)
    {
        return entry;
    }
    let looks = mob_looks(carry, resources().as_deref());
    if looks.is_empty() {
        return entry;
    }
    let mut entry = entry;
    if entry.status == Status::NotYet {
        match mods.install(carry) {
            Ok(_) => {
                entry.status = Status::Partly;
                entry.reason = "its looks for vanilla mobs run (lane 2)".into();
            }
            Err(why) => {
                entry.lane2 = Some(format!(
                    "{} looks for vanilla mobs, not installed: {why}",
                    looks.len()
                ));
                return entry;
            }
        }
    }
    let folders: BTreeSet<&str> = looks.iter().map(|(f, _)| f.as_str()).collect();
    let folders: Vec<&str> = folders.into_iter().collect();
    entry.lane2 = Some(match record_looks(game, &check.mod_id, &looks) {
        Ok(()) => format!(
            "{} looks for vanilla mobs ({}), on: each mob takes one or the vanilla look",
            looks.len(),
            folders.join(", ")
        ),
        Err(e) => format!("{} looks for vanilla mobs, not recorded: {e}", looks.len()),
    });
    entry
}

/// One jar on its own (a pack's own jar, or one the player imported): checked, converted,
/// and turned on when anything of it runs.
fn convert_one(file: &str, jar: &Path, scratch: &Path, mods: &mut Mods) -> Entry {
    let out = scratch.join(slug(file));
    let _ = fs::remove_dir_all(&out);
    let mut entry = match check::check(jar, &out) {
        Err(why) => Entry::plain(file, Status::NotYet, format!("can't be read: {why}")),
        Ok(c) if c.verdict == Verdict::No => {
            let e = Entry::checked(file, &c, Status::NotYet, c.summary.clone());
            let game = mods.game;
            lane2(game, &out.join("carry"), &c, mods, e)
        }
        Ok(c) => {
            creative_tab(&out.join("carry"), &c.mod_id, &c.name);
            let shaped = model_shapes(&out.join("carry"), &c);
            let e = installed(file, &c, mods.install(&out.join("carry")));
            let game = mods.game;
            with_shapes(lane2(game, &out.join("carry"), &c, mods, e), shaped)
        }
    };
    let _ = fs::remove_dir_all(&out);
    let names = [entry.mod_id.clone(), entry.name.clone()];
    cover(
        &mut entry,
        mods.game,
        &[names[0].as_str(), names[1].as_str()],
    );
    entry
}

/// Converts one mod's jar (named `name` where it came from) into `horizon-mods/`.
pub fn convert_jar(
    jar: &Path,
    name: &str,
    game: &Path,
    progress: Progress,
) -> Result<String, String> {
    progress(&format!("Converting {name}"), 0.3);
    let scratch = game.join(WORK).join(".mod");
    let mut mods = Mods::load(game);
    let entry = convert_one(name, jar, &scratch, &mut mods);
    let _ = fs::remove_dir_all(&scratch);
    let mut entries = [entry];
    if entries[0].status != Status::Works {
        let lane3 = [(0, name.to_owned(), jar.to_path_buf())];
        run_lane3(game, "", &lane3, &mut entries, progress, 0.5, 0.95);
    }
    let [mut entry] = entries;
    if let Ok(bytes) = fs::read(jar) {
        entry.sha1 = Some(import::sha1_hex(&bytes));
    }
    let title = if entry.version.is_empty() {
        entry.name.clone()
    } else {
        format!("{} {}", entry.name, entry.version)
    };
    progress("Converted", 1.0);
    let stem = name.strip_suffix(".jar").unwrap_or(name);
    finish(game, &slug(stem), &title, &[entry], &[])
}

/// Which records of a pack the converter carries over (lane 1): every mod and data pack that
/// has content, or may (one Modrinth didn't classify), whatever its code tier.
fn wanted(r: &inventory::Record) -> bool {
    (r.path.starts_with("mods/") || r.path.starts_with("datapacks/"))
        && matches!(r.class.as_str(), "content" | "unknown")
}

/// Which it downloads: those, and the mods whose Java code lane 3 may run (libraries other
/// mods' code needs, game-mechanics and server mods). Performance and client-interface mods
/// stay out: Horizon has its own.
fn downloaded(r: &inventory::Record) -> bool {
    wanted(r)
        || (r.path.starts_with("mods/")
            && matches!(r.class.as_str(), "library" | "other" | "server-only"))
}

/// Lane 3 for the jars of a conversion (`(entry index, file, jar)`): each made ready, then all
/// started together once; the entries get what came of it.
fn run_lane3(
    game: &Path,
    minecraft: &str,
    jars: &[(usize, String, PathBuf)],
    entries: &mut [Entry],
    progress: Progress,
    from: f32,
    to: f32,
) {
    if jars.is_empty() {
        return;
    }
    if !crate::lane3::ready() {
        for (i, ..) in jars {
            entries[*i].lane3 = Some("not available in this app".into());
        }
        return;
    }
    let mut lazy = crate::lane3::Lazy::new(game, minecraft);
    let mut prepared: Vec<(usize, crate::lane3::Prepared)> = Vec::new();
    for (k, (i, file, jar)) in jars.iter().enumerate() {
        progress(
            &format!("Java code (lane 3): {file} ({}/{})", k + 1, jars.len()),
            from + (to - from) * (k as f32 / jars.len() as f32),
        );
        match crate::lane3::prepare(jar, file, game, &mut lazy) {
            crate::lane3::Outcome::Ready(p) => prepared.push((*i, p)),
            crate::lane3::Outcome::Skipped(why) => {
                entries[*i].lane3 = Some(format!("not run: {why}"))
            }
            crate::lane3::Outcome::Failed(why) => {
                entries[*i].lane3 = Some(format!("not run: {why}"))
            }
        }
    }
    if prepared.is_empty() {
        return;
    }
    progress("Java code (lane 3): starting it", to);
    let all: Vec<crate::lane3::Prepared> = prepared.iter().map(|(_, p)| p.clone()).collect();
    match crate::lane3::check(game, &all) {
        Ok(results) => {
            for (i, p) in &prepared {
                let started = results
                    .iter()
                    .find(|(id, _)| *id == p.id)
                    .map(|(_, s)| s.clone());
                lane3_outcome(game, &mut entries[*i], p, started);
            }
        }
        Err(why) => {
            // Kept and on: the game starts and checks them when a world opens.
            for (i, p) in &prepared {
                let _ = browser::java::set_enabled(game, &p.id, true);
                entries[*i].lane3 = Some(format!("made ready, not checked ({why})"));
            }
        }
    }
}

/// An entry once lane 3 started its code: a mod whose code runs is turned on; one that stopped
/// is taken out again (converting it after an update tries again).
fn lane3_outcome(
    game: &Path,
    entry: &mut Entry,
    p: &crate::lane3::Prepared,
    started: Option<crate::lane3::Started>,
) {
    use crate::lane3::Started;
    let mixins = if p.mixins > 0 {
        format!(
            "; {} Mixin config(s) not applied: they patch vanilla's own code (lane 4)",
            p.mixins
        )
    } else {
        String::new()
    };
    let remove = || {
        let _ = fs::remove_dir_all(game.join(crate::lane3::DIR).join(&p.id));
    };
    match started {
        Some(Started::Ran(events)) => {
            let _ = browser::java::set_enabled(game, &p.id, true);
            entry.lane3 = Some(format!("its Java code runs and is on: {events}{mixins}"));
            if entry.status == Status::NotYet {
                entry.status = Status::Partly;
                entry.reason = format!("its Java code runs in lane 3 ({events})");
            }
        }
        Some(Started::Missing(api)) => {
            remove();
            entry.lane3 = Some(format!(
                "its code stopped at {api}, which Horizon's Java API doesn't have yet{mixins}"
            ));
            entry.missing_api = Some(api);
        }
        Some(Started::Failed(why)) => {
            remove();
            entry.lane3 = Some(format!("its code threw: {why}{mixins}"));
        }
        None => {
            remove();
            entry.lane3 = Some("its code didn't start".into());
        }
    }
}

/// Converts a modpack file (`.mrpack`, a CurseForge zip): its mods downloaded and carried over
/// together into `horizon-mods/`, its resource and shader packs copied for the game.
/// `resources` is the prepared vanilla version, which limits block templates to the game's.
pub fn convert_pack(
    pack_file: &Path,
    game: &Path,
    resources: Option<&Path>,
    progress: Progress,
) -> Result<String, String> {
    let pack = import::import(pack_file)?;
    let name = if pack.name.is_empty() || pack.name == "modlist" {
        pack_file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "modpack".into())
    } else {
        pack.name.clone()
    };
    let work = game.join(WORK).join(slug(&name));
    fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    // The steps' lines, which also move the bar: fetch says "i/n file" before each download,
    // carry one line per mod.
    let phase = Cell::new(0u8);
    let (done, total) = (Cell::new(0usize), Cell::new(1usize));
    let log = |line: String| {
        log::info!("modpack: {line}");
        match phase.get() {
            1 => {
                if let Some((i, n)) = line
                    .split_once(' ')
                    .and_then(|(count, _)| count.split_once('/'))
                    .and_then(|(i, n)| Some((i.parse::<f32>().ok()?, n.parse::<f32>().ok()?)))
                {
                    progress(
                        &format!("Downloading mods ({i}/{n})"),
                        0.1 + 0.45 * (i / n.max(1.0)),
                    );
                }
            }
            2 if line.contains(" files, ") => {
                done.set(done.get() + 1);
                let share = done.get() as f32 / total.get().max(1) as f32;
                progress(
                    &format!("Converting mods ({}/{})", done.get(), total.get()),
                    0.6 + 0.3 * share.min(1.0),
                );
            }
            _ => {}
        }
    };
    progress(&format!("Reading {name}"), 0.02);
    pipeline::import(pack_file, &work, &log)?;
    progress("Asking Modrinth about its files", 0.05);
    let records = pipeline::inventory(&work, false, &log)?;
    let mut notes = Vec::new();
    if let Err(e) = resourcepacks::fetch(&work, &log) {
        notes.push(format!("Resource packs not downloaded: {e}"));
    }
    phase.set(1);
    let shaders = |r: &inventory::Record| r.path.starts_with("shaderpacks/");
    let fetched = fetch::fetch(
        &work,
        &records,
        None,
        &|r: &inventory::Record| downloaded(r) || shaders(r),
        &log,
    )?;
    // With the jars here, they tell classes and tiers better than Modrinth alone.
    progress("Reading the jars", 0.56);
    let records = pipeline::inventory(&work, true, &log)?;
    let only: Vec<String> = records
        .iter()
        .filter(|r| wanted(r) && r.jar.is_some())
        .map(|r| r.key.clone())
        .collect();
    phase.set(2);
    total.set(only.len());
    if !only.is_empty() {
        pipeline::carry(&work, resources, &only, &log)?;
    }
    phase.set(0);
    let rows: BTreeMap<String, Row> = pipeline::report(&work, &log)?
        .into_iter()
        .map(|r| (r.key.clone(), r))
        .collect();
    progress("Turning the mods on", 0.92);
    let files: BTreeMap<&str, &import::PackFile> =
        pack.files.iter().map(|f| (f.path.as_str(), f)).collect();
    let outcomes: BTreeMap<&str, &fetch::Outcome> =
        fetched.files.iter().map(|o| (o.key.as_str(), o)).collect();
    let mut mods = Mods::load(game);
    let mut entries = Vec::new();
    let mut lane3_jars: Vec<(usize, String, PathBuf)> = Vec::new();
    for r in &records {
        let kind = r.path.split('/').next().unwrap_or("");
        let file = r.path.rsplit('/').next().unwrap_or(&r.path);
        let mut entry = if kind == "resourcepacks" {
            // Copied below with the pack's own; listed in the notes.
            continue;
        } else if shaders(r) {
            shader_pack(file, &work.join("jars").join(file), game)
        } else if let (true, Some(jar), Some(info)) = (only.contains(&r.key), &r.jar, &r.info) {
            let out = work.join("mods").join(&r.key);
            match pipeline::read_json(&out.join("carry-report.json")).and_then(
                |report: horizon_port_pack::carry::Report| check::assess(jar, info, &report),
            ) {
                Ok(c) if c.verdict == Verdict::No => {
                    let e = Entry::checked(file, &c, Status::NotYet, c.summary.clone());
                    lane2(game, &out.join("carry"), &c, &mut mods, e)
                }
                Ok(c) => {
                    creative_tab(&out.join("carry"), &c.mod_id, &c.name);
                    let shaped = model_shapes(&out.join("carry"), &c);
                    let e = installed(file, &c, mods.install(&out.join("carry")));
                    with_shapes(lane2(game, &out.join("carry"), &c, &mut mods, e), shaped)
                }
                Err(why) => Entry::plain(
                    file,
                    Status::NotYet,
                    format!("could not be carried over: {why}"),
                ),
            }
        } else if let Some(o) = outcomes
            .get(r.key.as_str())
            .filter(|o| matches!(o.how, How::Failed | How::NeedsKey | How::NotFound))
        {
            let mut why = o.how.label().to_owned();
            if !o.detail.is_empty() {
                why = format!("{why}: {}", o.detail);
            }
            Entry::plain(file, Status::NotDownloaded, why)
        } else {
            match rows.get(&r.key) {
                Some(row) if row.status == CoverageStatus::NotNeeded => {
                    Entry::plain(file, Status::NotNeeded, row.reason.clone())
                }
                Some(row) => Entry::plain(file, Status::NotYet, row.reason.clone()),
                None => Entry::plain(file, Status::NotYet, "not classified"),
            }
        };
        if entry.mod_id.is_empty() {
            entry.name = if r.name.is_empty() {
                file.to_owned()
            } else {
                r.name.clone()
            };
            entry.version = r.version.clone();
            entry.mod_id = r.key.clone();
            entry.license = r.license.clone();
        }
        if let Some(f) = files.get(r.path.as_str()) {
            entry.sha1 = f.sha1.clone();
            entry.links = f.downloads.clone();
        }
        // A known mod plays in Horizon's own version, downloaded or not.
        let names = [r.key.clone(), entry.mod_id.clone(), r.name.clone()];
        cover(
            &mut entry,
            game,
            &[names[0].as_str(), names[1].as_str(), names[2].as_str()],
        );
        if kind == "mods"
            && entry.status != Status::Works
            && let Some(jar) = r.jar.clone().filter(|j| j.is_file())
        {
            lane3_jars.push((entries.len(), file.to_owned(), jar));
        }
        entries.push(entry);
    }
    // The jars a pack ships itself (not downloads), each converted on its own.
    let own = work.join("own");
    let _ = fs::remove_dir_all(&own);
    match import::extract_overrides(pack_file, "mods", &own) {
        Ok(jars) => {
            for jar in jars
                .iter()
                .filter(|j| j.extension().is_some_and(|e| e == "jar"))
            {
                let file = jar.file_name().unwrap_or_default().to_string_lossy();
                let mut entry = convert_one(&file, jar, &work.join("checks"), &mut mods);
                entry.file = format!("{file} (shipped in the pack)");
                if let Ok(bytes) = fs::read(jar) {
                    entry.sha1 = Some(import::sha1_hex(&bytes));
                }
                if entry.status != Status::Works {
                    lane3_jars.push((entries.len(), file.to_string(), jar.clone()));
                }
                entries.push(entry);
            }
        }
        Err(e) => notes.push(format!(
            "The pack's own mods (overrides/mods) not read: {e}"
        )),
    }
    match import::extract_overrides(pack_file, "shaderpacks", &own.join("shaderpacks")) {
        Ok(packs) => {
            for p in packs {
                let file = p
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                entries.push(shader_pack(&file, &p, game));
            }
        }
        Err(e) => notes.push(format!("The pack's own shader packs not read: {e}")),
    }
    run_lane3(
        game,
        &pack.minecraft,
        &lane3_jars,
        &mut entries,
        progress,
        0.93,
        0.96,
    );
    progress("Copying the resource packs", 0.96);
    match resourcepacks::install(&work, game) {
        Ok(report) => {
            for (path, o) in &report.files {
                notes.push(format!("{path}: {:?}, {}", o.state, o.detail));
            }
            if !report.files.is_empty() {
                notes.push("Turn the resource packs on in Options > Resource Packs.".into());
            }
        }
        Err(e) => notes.push(format!("Resource packs not copied: {e}")),
    }
    for (folder, count) in &pack.overrides {
        if !matches!(folder.as_str(), "mods" | "shaderpacks" | "resourcepacks") {
            notes.push(format!(
                "{count} files in overrides/{folder}/: not converted ({})",
                match folder.as_str() {
                    "config" | "defaultconfigs" => "Java mods' settings",
                    "kubejs" => "KubeJS scripts",
                    "datapacks" | "global_packs" => "data packs",
                    _ => "not read by Horizon",
                }
            ));
        }
    }
    // A big pack's jars would fill the phone: they go once converted (the prompt names each).
    let _ = fs::remove_dir_all(work.join("jars"));
    let _ = fs::remove_dir_all(work.join("mods"));
    let _ = fs::remove_dir_all(&own);
    let _ = fs::remove_dir_all(work.join("checks"));
    progress("Done", 1.0);
    finish(
        game,
        &slug(&name),
        &pipeline::title(&pack),
        &entries,
        &notes,
    )
}

/// A pack's shader pack, copied into `shaderpacks/` when the player has none of that name.
fn shader_pack(file: &str, from: &Path, game: &Path) -> Entry {
    if !from.is_file() {
        return Entry::plain(file, Status::NotDownloaded, "shader pack not downloaded");
    }
    let dir = game.join("shaderpacks");
    let dest = dir.join(file);
    let copied = if dest.exists() {
        Ok("already in Shader Packs")
    } else {
        fs::create_dir_all(&dir)
            .and_then(|()| fs::copy(from, &dest))
            .map(|_| "copied into Shader Packs")
            .map_err(|e| e.to_string())
    };
    match copied {
        Ok(what) => Entry::plain(
            file,
            Status::Partly,
            format!(
                "{what}; Horizon runs part of the OptiFine/Iris format, pick it in Video Settings"
            ),
        ),
        Err(e) => Entry::plain(file, Status::NotDownloaded, format!("not copied: {e}")),
    }
}

/// Downloads the newest version of Modrinth modpack `slug` and converts it.
pub fn convert_modrinth_pack(
    slug_or_id: &str,
    game: &Path,
    resources: Option<&Path>,
    progress: Progress,
) -> Result<String, String> {
    let source = browser::source(browser::SourceId::Modrinth).ok_or("Modrinth is not available")?;
    progress("Finding the modpack on Modrinth", 0.01);
    let project = source.project(slug_or_id)?;
    let versions = source.versions(&project)?;
    let usable = |v: &&browser::Version| v.blocked.is_none() && v.file().is_some();
    let version = versions
        .iter()
        .filter(usable)
        .find(|v| {
            v.loaders
                .iter()
                .any(|l| browser::java::LOADERS.contains(&l.as_str()))
        })
        .or_else(|| versions.iter().find(usable))
        .ok_or_else(|| format!("{} has no version to download.", project.title))?;
    let file = version.file().ok_or("The version has no file")?;
    let folder = game.join(WORK).join(".downloads");
    let _ = fs::remove_dir_all(&folder);
    let downloaded = source.fetch(file, &folder, &|f| {
        progress(
            &format!("Downloading {} {}", project.title, version.number),
            0.01 + 0.01 * f,
        )
    })?;
    let result = convert_pack(&downloaded, game, resources, progress);
    let _ = fs::remove_dir_all(&folder);
    result
}

/// Writes the report and prompt beside the mods and returns the launcher's line.
/// The mods a conversion couldn't turn on, kept beside the mods so the launcher's Installed
/// lists them too (with why), not only the report: `key\tname\tversion\tstatus\twhy\treport`.
pub const CONVERTED: &str = ".converted.tsv";

fn record_not_running(dir: &Path, report: &str, entries: &[Entry]) {
    let path = dir.join(CONVERTED);
    let clean = |s: &str| s.replace(['\t', '\n'], " ");
    let mut lines: Vec<String> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    for e in entries {
        let key = if e.mod_id.is_empty() {
            &e.file
        } else {
            &e.mod_id
        };
        let prefix = format!("{}\t", clean(key));
        lines.retain(|l| !l.starts_with(&prefix));
        let status = match e.status {
            Status::NotYet => "not yet",
            Status::Off => "not turned on",
            Status::NotDownloaded => "not downloaded",
            _ => continue,
        };
        lines.push(format!(
            "{}\t{}\t{}\t{status}\t{}\t{report}",
            clean(key),
            clean(&e.name),
            clean(&e.version),
            clean(&e.reason)
        ));
    }
    let _ = fs::write(&path, lines.join("\n") + "\n");
}

fn finish(
    game: &Path,
    stem: &str,
    title: &str,
    entries: &[Entry],
    notes: &[String],
) -> Result<String, String> {
    let dir = game.join(browser::java::MODS_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = format!("{stem}-para-o-Claude.txt");
    let path = dir.join(&name);
    fs::write(&path, document(title, entries, notes)).map_err(|e| e.to_string())?;
    record_not_running(&dir, &name, entries);
    let count = |s: Status| entries.iter().filter(|e| e.status == s).count();
    let missing = entries.iter().filter(|e| e.status.missing()).count();
    let java = entries
        .iter()
        .filter(|e| {
            e.lane3
                .as_deref()
                .is_some_and(|l| l.starts_with("its Java code runs"))
        })
        .count();
    Ok(format!(
        "{title}: {} work, {} partly ({java} with their Java code running), {} not yet, {} not \
         needed. {} Open Play to load them.",
        count(Status::Works),
        count(Status::Partly),
        count(Status::NotYet) + count(Status::Off) + count(Status::NotDownloaded),
        count(Status::NotNeeded),
        if missing == 0 {
            "Everything converted.".to_owned()
        } else {
            format!("What is missing, and the prompt to finish it: Files > Mods > {name}.")
        }
    ))
}

/// The file: the report for the player, then the prompt for the ports left.
fn document(title: &str, entries: &[Entry], notes: &[String]) -> String {
    let count = |s: Status| entries.iter().filter(|e| e.status == s).count();
    let mut out = format!(
        "HORIZON MOBILE — CONVERSÃO DE {title}\n\
         Convertido para o Minecraft 26.3 do Horizon (Horizon Android {}).\n\n\
         Funciona: {}   Parcial: {}   Ainda não: {}   Não ligado: {}   Não baixado: {}   \
         Não precisa: {}\n\n\
         O que é dado (blocos, itens, receitas, loot, tags, modelos, texturas, sons, traduções, \
         estruturas, geração de mundo) foi convertido e ligado junto com os mods do Horizon. O \
         que é código Java (IA de mobs, máquinas, efeitos, mixins) ainda não roda: está listado \
         abaixo, mod por mod. Para chegar a 100%, mande este arquivo inteiro para o Claude; o \
         pedido para ele está no fim.\n",
        crate::backend::VERSION,
        count(Status::Works),
        count(Status::Partly),
        count(Status::NotYet),
        count(Status::Off),
        count(Status::NotDownloaded),
        count(Status::NotNeeded),
    );
    let label = |e: &Entry| {
        let mut s = e.name.clone();
        if !e.version.is_empty() {
            s.push(' ');
            s.push_str(&e.version);
        }
        if e.file != e.name {
            s.push_str(&format!(" ({})", e.file));
        }
        s
    };
    for status in Status::ALL {
        let list: Vec<&Entry> = entries.iter().filter(|e| e.status == status).collect();
        if list.is_empty() {
            continue;
        }
        out.push_str(&format!("\n== {} — {} ==\n", status.heading(), list.len()));
        for e in list {
            out.push_str(&format!("- {}: {}\n", label(e), e.reason));
            if !e.ports.is_empty() && status != Status::NotNeeded {
                out.push_str(&format!("    convertido: {}\n", e.ports.join(", ")));
            }
            for l in &e.left {
                out.push_str(&format!("    falta: {l}\n"));
            }
            if let Some(l2) = &e.lane2 {
                out.push_str(&format!("    comportamento como dado (faixa 2): {l2}\n"));
            }
            if let Some(l3) = &e.lane3 {
                out.push_str(&format!("    código Java (faixa 3): {l3}\n"));
            }
        }
    }
    if !notes.is_empty() {
        out.push_str("\n== O RESTO DO MODPACK ==\n");
        for n in notes {
            out.push_str(&format!("- {n}\n"));
        }
    }
    let missing: Vec<&Entry> = entries.iter().filter(|e| e.status.missing()).collect();
    out.push_str(
        "\n\n==================================================================\n\
         PEDIDO PARA O CLAUDE (mande o arquivo inteiro, ou copie daqui até o fim)\n\
         ==================================================================\n\n",
    );
    if missing.is_empty() && notes.is_empty() {
        out.push_str(&format!(
            "Claude: the Horizon Mobile converter converted all of {title}; nothing is left \
             to port. Check it plays as in Java if you like.\n"
        ));
        return out;
    }
    out.push_str(&format!(
        "Claude: this is the Horizon Mobile converter's report for {title}. Bring it to 100% \
         in Horizon Mobile (repository felipeminecraftorg-max/horizon-android, branch \
         `android`; follow its CLAUDE.md). The converter already carried over every mod's \
         data (blocks, items, recipes, loot, tags, assets, worldgen) into `horizon-mods/` on \
         the phone. What is left is what only Java code does, below, mod by mod.\n\n\
         For each mod: get the jar (Modrinth by its SHA-1, or the link), find what each listed \
         class does from the mod's behaviour, data, wiki and, for open-licence mods only, its \
         source; then add it to Horizon in Rust: as a known-mod port or a reusable pattern \
         (entity, block entity, item use, worldgen feature, processor, event) that the \
         converter applies to every jar that has it, so the next conversion gets it without \
         you. Each mod names the lanes that would cover it: lane 2, behaviour as data (one \
         Rust version of a vanilla building block serves every mod that uses it); lane 3, \
         behaviour callbacks; lane 4, a known-mod port (`KNOWN` in android/mod_convert.rs and \
         game/android_bundled/compat.rs), which the converter turns on by itself next time. \
         Prefer the lane that covers the most mods. Keep the vanilla look by default, write tests with signal, and when a mod \
         can't be matched exactly, adapt it and say how. Then build the Android dev APK and \
         tell me, in simple Portuguese, what changed and how to convert again.\n\n\
         {} mods or files are left:\n",
        missing.len()
    ));
    for e in missing {
        out.push_str(&format!("\n### {}\n", label(e)));
        let mut facts = Vec::new();
        if !e.mod_id.is_empty() {
            facts.push(format!("id `{}`", e.mod_id));
        }
        if !e.loader.is_empty() {
            facts.push(e.loader.clone());
        }
        if !e.license.is_empty() {
            facts.push(format!("licence {}", e.license));
        }
        if !facts.is_empty() {
            out.push_str(&format!("- {}\n", facts.join(", ")));
        }
        if let Some(sha1) = &e.sha1 {
            out.push_str(&format!("- jar SHA-1 {sha1}\n"));
        }
        for link in &e.links {
            out.push_str(&format!("- download {link}\n"));
        }
        out.push_str(&format!("- state: {}\n", e.reason));
        if !e.ports.is_empty() {
            out.push_str(&format!("- converted: {}\n", e.ports.join(", ")));
        }
        for l in &e.left {
            out.push_str(&format!("- left out: {l}\n"));
        }
        if !e.events.is_empty() {
            let events: Vec<&str> = e.events.iter().take(20).map(String::as_str).collect();
            out.push_str(&format!(
                "- game events it handles: {}\n",
                events.join(", ")
            ));
        }
        for ex in &e.examples {
            out.push_str(&format!("- class {ex}\n"));
        }
        for lane in lanes(e) {
            out.push_str(&format!("- lane: {lane}\n"));
        }
        if let Some(l2) = &e.lane2 {
            out.push_str(&format!("- lane 2 (behaviour as data): {l2}\n"));
        }
        if let Some(l3) = &e.lane3 {
            out.push_str(&format!("- lane 3 (its Java code on ART): {l3}\n"));
        }
    }
    let mut apis: BTreeMap<&str, usize> = BTreeMap::new();
    for e in entries {
        if let Some(api) = &e.missing_api {
            *apis.entry(api.as_str()).or_default() += 1;
        }
    }
    if !apis.is_empty() {
        let mut ranked: Vec<(&str, usize)> = apis.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        out.push_str(
            "\n### Lane 3: what Horizon's Java API lacks, most needed first\n\
             Add each to the API slice (packaging/android/java: net/minecraft, net/fabricmc), \
             backed by the game through dev.horizon.android.lane3.Native \
             (game/android_bundled/lane3.rs); then converting again runs those mods further.\n",
        );
        for (api, n) in ranked {
            out.push_str(&format!("- {api}: {n} mod(s) stop there\n"));
        }
    }
    if !notes.is_empty() {
        out.push_str("\n### The rest of the pack\n");
        for n in notes {
            out.push_str(&format!("- {n}\n"));
        }
    }
    out
}

/// The lanes that would cover what an entry's code does.
fn lanes(e: &Entry) -> Vec<&'static str> {
    use horizon_port_pack::classes::Feature as F;
    let mut out = BTreeSet::new();
    for f in &e.features {
        out.insert(match f {
            F::Entity | F::Renderer => {
                "2: entities as data (attributes, goals, spawns, models, animations); Horizon \
                 needs mod entity types first"
            }
            F::BlockEntity | F::Menu | F::Screen => {
                "3: machines (block entities, menus, screens: mod API 0.4)"
            }
            F::Mixin | F::Unread => "4: a native port (Mixins patch vanilla's own code)",
            F::Worldgen => "2: the Java worldgen type, once in Rust for every mod that uses it",
            F::BlockShape => "2: block shapes as data",
            _ => "3: behaviour callbacks (use, ticks, events), or 4 when it is a common mechanic",
        });
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modrinth_modpack_links_name_their_pack() {
        for (query, slug) in [
            (
                "https://modrinth.com/modpack/krabs-modpack",
                Some("krabs-modpack"),
            ),
            (
                "https://modrinth.com/modpack/krabs-modpack/versions",
                Some("krabs-modpack"),
            ),
            ("modrinth.com/modpack/abc?x=1", Some("abc")),
            ("modpack:my_pack", Some("my_pack")),
            ("https://modrinth.com/mod/sodium", None),
            ("farmers delight", None),
        ] {
            assert_eq!(modpack_slug(query).as_deref(), slug, "{query}");
        }
    }

    #[test]
    fn known_mods_are_found_by_id_slug_or_name() {
        assert_eq!(known(&["veinminer"]).map(|k| k.1), Some("veinminer"));
        assert_eq!(
            known(&["", "Tree Harvester"]).map(|k| k.1),
            Some("treeharvester")
        );
        assert_eq!(
            known(&["right-click-harvest"]).map(|k| k.1),
            Some("rightclickharvest")
        );
        assert!(known(&["sodium", "Sodium"]).is_none());
        // Every one is a built-in mod the mod list knows.
        for (_, id, ..) in KNOWN.iter() {
            assert!(
                horizon_client::mod_list::ANDROID_BUNDLED
                    .iter()
                    .any(|(b, ..)| *b == *id)
            );
        }
    }

    #[test]
    fn the_prompt_asks_only_for_what_is_missing() {
        let mut partly = Entry::plain("ruby.jar", Status::Partly, "blocks port; not entities");
        partly.name = "Ruby".into();
        partly.left = vec!["2 entities".into()];
        partly.sha1 = Some("ab".repeat(20));
        let mut works = Entry::plain("gems.jar", Status::Works, "content only");
        works.name = "Gems".into();
        let text = document("Pack", &[works, partly], &[]);
        let (report, prompt) = text.split_once("PEDIDO PARA O CLAUDE").unwrap();
        assert!(report.contains("Funciona: 1   Parcial: 1"), "{report}");
        assert!(report.contains("falta: 2 entities"));
        assert!(prompt.contains("### Ruby (ruby.jar)"), "{prompt}");
        assert!(prompt.contains(&format!("jar SHA-1 {}", "ab".repeat(20))));
        assert!(!prompt.contains("Gems"), "{prompt}");
    }

    #[test]
    fn a_converted_mod_never_takes_a_namespace_from_one_already_on() {
        let game = std::env::temp_dir().join(format!("mod-convert-{}", std::process::id()));
        let _ = fs::remove_dir_all(&game);
        let carry = |id: &str, namespaces: &str| {
            let dir = game.join("carries").join(id);
            fs::create_dir_all(dir.join("data")).unwrap();
            fs::write(
                dir.join(horizon_mods::manifest::MANIFEST),
                format!(
                    "id = \"{id}\"\nname = \"{id}\"\nversion = \"1\"\napi = \"0.2\"\n\
                     capabilities = [\"registries\"]\nnamespaces = [{namespaces}]\n"
                ),
            )
            .unwrap();
            dir
        };
        let mut mods = Mods::load(&game);
        assert_eq!(
            mods.install(&carry("dungeons", "\"yungsapi\"")).unwrap(),
            "dungeons"
        );
        // A new conversion of the same mod replaces the old one.
        assert!(mods.install(&carry("dungeons", "\"yungsapi\"")).is_ok());
        let mut again = Mods::load(&game);
        let err = again
            .install(&carry("fortresses", "\"yungsapi\""))
            .unwrap_err();
        assert!(
            err.contains("yungsapi") && err.contains("dungeons"),
            "{err}"
        );
        assert!(game.join("horizon-mods/dungeons/data").is_dir());
        assert!(!game.join("horizon-mods/fortresses").exists());
        assert!(browser::java::is_enabled(&game, "dungeons"));
        fs::remove_dir_all(&game).unwrap();
    }
}
