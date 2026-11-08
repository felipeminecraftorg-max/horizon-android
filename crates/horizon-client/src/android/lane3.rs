//! Lane 3 of the mod converter: a Java mod's own code, run by Android's runtime (ART). The Java
//! half is `packaging/android/java/dev/horizon/android/lane3/` (D8 to make DEX on the phone, the
//! loader, and the slice of Fabric's and Minecraft's API Horizon answers); this is the Rust half
//! the converter and the game call.
//!
//! Converting: a Fabric jar (and the library jars it carries) is renamed from Fabric's
//! intermediary names to Mojang's with the player's own download of the mappings, turned into
//! DEX, and started once as a check, which tells how far its code runs and what it missed. Each
//! mod is kept in `lane3/<mod id>/` beside `options.txt` and turned on in the mod list; the game
//! starts the ones on when a world opens (`game/android_bundled/lane3.rs`).
use horizon_port_pack::remap::{self, Mappings};
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JClass, JObject, JString, JValue},
};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Where lane 3 keeps its mods and mappings, beside `options.txt`.
pub const DIR: &str = "lane3";

static VM: OnceLock<JavaVM> = OnceLock::new();
static CLASS: OnceLock<GlobalRef> = OnceLock::new();

/// Finds the Java half through the activity's class loader (a thread the runtime didn't start
/// can't find the app's classes by name).
///
/// # Safety
/// `vm` and `activity` come from android-activity's onCreate and are valid during the call.
pub unsafe fn initialize(
    vm: *mut jni::sys::JavaVM,
    activity: jni::sys::jobject,
) -> Result<(), String> {
    if CLASS.get().is_some() {
        return Ok(());
    }
    let vm = unsafe { JavaVM::from_raw(vm) }.map_err(|e| e.to_string())?;
    let class = {
        let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let found = env.with_local_frame(16, |env| -> jni::errors::Result<GlobalRef> {
            let activity = unsafe { JObject::from_raw(activity) };
            let loader = env
                .call_method(
                    &activity,
                    "getClassLoader",
                    "()Ljava/lang/ClassLoader;",
                    &[],
                )?
                .l()?;
            let name = JObject::from(env.new_string("dev.horizon.android.lane3.Lane3")?);
            let class = env
                .call_method(
                    &loader,
                    "loadClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    &[JValue::Object(&name)],
                )?
                .l()?;
            env.new_global_ref(class)
        });
        if found.is_err() {
            let _ = env.exception_clear();
        }
        found.map_err(|e| e.to_string())?
    };
    let _ = CLASS.set(class);
    let _ = VM.set(vm);
    Ok(())
}

/// Whether the Java half is there.
pub fn ready() -> bool {
    VM.get().is_some() && CLASS.get().is_some()
}

/// Calls the Java half on this thread (attached for good: the server thread calls every tick).
fn call<T>(f: impl FnOnce(&mut JNIEnv, &JClass) -> jni::errors::Result<T>) -> Result<T, String> {
    let vm = VM.get().ok_or("lane 3 is not available in this app")?;
    let class = CLASS.get().ok_or("lane 3 is not available in this app")?;
    let mut env = vm
        .attach_current_thread_permanently()
        .map_err(|e| e.to_string())?;
    let result = env.with_local_frame(64, |env| -> jni::errors::Result<T> {
        let class = JClass::from(env.new_local_ref(class.as_obj())?);
        f(env, &class)
    });
    if result.is_err() && env.exception_check().unwrap_or(false) {
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
    result.map_err(|e| e.to_string())
}

fn string(env: &mut JNIEnv, value: JObject) -> jni::errors::Result<String> {
    if value.is_null() {
        return Ok(String::new());
    }
    let s = JString::from(value);
    Ok(env.get_string(&s)?.into())
}

/// A static method of the Java half taking strings and answering one.
fn strings(method: &str, args: &[&str]) -> Result<String, String> {
    let sig = format!(
        "({})Ljava/lang/String;",
        "Ljava/lang/String;".repeat(args.len())
    );
    call(|env, class| {
        let mut held = Vec::new();
        for a in args {
            held.push(JObject::from(env.new_string(a)?));
        }
        let values: Vec<JValue> = held.iter().map(JValue::Object).collect();
        let out = env.call_static_method(class, method, &sig, &values)?.l()?;
        string(env, out)
    })
}

/// Jars to one DEX archive (D8 on the phone).
pub fn dex(inputs: &[PathBuf], output: &Path) -> Result<(), String> {
    let list: Vec<String> = inputs.iter().map(|p| p.display().to_string()).collect();
    let _ = fs::remove_file(output);
    let why = strings("dex", &[&list.join("\n"), &output.display().to_string()])?;
    if why.is_empty() && output.is_file() {
        Ok(())
    } else if why.is_empty() {
        Err("D8 wrote nothing".into())
    } else {
        Err(why)
    }
}

/// How one mod's code started.
#[derive(Clone, Debug, PartialEq)]
pub enum Started {
    /// Its entrypoints ran: what it registered.
    Ran(String),
    /// It stopped at a class or member Horizon's API slice lacks.
    Missing(String),
    /// It threw something else.
    Failed(String),
}

/// Starts mods (`(id, entrypoint class)`, libraries first) from their DEX archives in one class
/// loader; `dir` is their folder (`FabricLoader.getConfigDir()` under it).
pub fn start(
    dex: &[PathBuf],
    entries: &[(String, String)],
    dir: &Path,
) -> Result<Vec<(String, Started)>, String> {
    let dex: Vec<String> = dex.iter().map(|p| p.display().to_string()).collect();
    let mods: Vec<String> = entries.iter().map(|(id, c)| format!("{id}\t{c}")).collect();
    let report = strings(
        "start",
        &[
            &dex.join("\n"),
            &mods.join("\n"),
            &dir.display().to_string(),
        ],
    )?;
    let mut out = Vec::new();
    for line in report.lines() {
        let mut parts = line.splitn(3, '\t');
        let (Some(id), Some(state)) = (parts.next(), parts.next()) else {
            continue;
        };
        let detail = parts.next().unwrap_or("").to_owned();
        out.push((
            id.to_owned(),
            match state {
                "ok" => Started::Ran(detail),
                "missing" => Started::Missing(detail),
                _ => Started::Failed(detail),
            },
        ));
    }
    Ok(out)
}

pub fn use_block(pos: [i32; 3], face: i32, hand: i32) -> i32 {
    call(|env, class| {
        env.call_static_method(
            class,
            "useBlock",
            "(IIIII)I",
            &[
                JValue::Int(pos[0]),
                JValue::Int(pos[1]),
                JValue::Int(pos[2]),
                JValue::Int(face),
                JValue::Int(hand),
            ],
        )?
        .i()
    })
    .unwrap_or(0)
}

pub fn before_break(pos: [i32; 3], state: &str) -> bool {
    call(|env, class| {
        let s = JObject::from(env.new_string(state)?);
        env.call_static_method(
            class,
            "beforeBreak",
            "(IIILjava/lang/String;)Z",
            &[
                JValue::Int(pos[0]),
                JValue::Int(pos[1]),
                JValue::Int(pos[2]),
                JValue::Object(&s),
            ],
        )?
        .z()
    })
    .unwrap_or(true)
}

pub fn after_break(pos: [i32; 3], state: &str) {
    let _ = call(|env, class| {
        let s = JObject::from(env.new_string(state)?);
        env.call_static_method(
            class,
            "afterBreak",
            "(IIILjava/lang/String;)V",
            &[
                JValue::Int(pos[0]),
                JValue::Int(pos[1]),
                JValue::Int(pos[2]),
                JValue::Object(&s),
            ],
        )?;
        Ok(())
    });
}

pub fn server_tick(end: bool) {
    let _ = call(|env, class| {
        env.call_static_method(class, "serverTick", "(Z)V", &[JValue::Bool(end.into())])?;
        Ok(())
    });
}

/// What mods threw during play, a line each.
pub fn errors() -> String {
    strings("errors", &[]).unwrap_or_default()
}

pub fn stop() {
    let _ = call(|env, class| {
        env.call_static_method(class, "stop", "()V", &[])?;
        Ok(())
    });
}

// ---------------------------------------------------------------------------------------------
// Converting

/// The Minecraft version a pack or mod names (`~1.21.1`, `>=1.21` → `1.21.1`, `1.21`).
pub fn minecraft_version(asked: &str) -> String {
    let v: String = asked
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let v = v.trim_end_matches('.');
    if v.is_empty() {
        "1.21.1".into()
    } else {
        v.to_owned()
    }
}

fn get(url: &str, max: u64) -> Result<Vec<u8>, String> {
    let agent = ureq::AgentBuilder::new()
        .https_only(true)
        .timeout_connect(std::time::Duration::from_secs(20))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();
    let mut bytes = Vec::new();
    agent
        .get(url)
        .set("User-Agent", "Horizon-Android-mod-converter")
        .call()
        .map_err(|e| format!("{url}: {e}"))?
        .into_reader()
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("{url}: {e}"))?;
    if bytes.len() as u64 > max {
        return Err(format!("{url}: too large"));
    }
    Ok(bytes)
}

/// Fabric's intermediary names to Mojang's for Minecraft `version`: from the player's own
/// downloads (Fabric's Maven, Mojang's servers), joined once and kept in `lane3/`.
pub fn mappings(game: &Path, version: &str) -> Result<Mappings, String> {
    let dir = game.join(DIR);
    let kept = dir.join(format!("mappings-{version}.tsv"));
    if let Ok(text) = fs::read_to_string(&kept) {
        let m = Mappings::load(&text);
        if !m.classes.is_empty() {
            return Ok(m);
        }
    }
    let jar = get(
        &format!(
            "https://maven.fabricmc.net/net/fabricmc/intermediary/{version}/intermediary-{version}-v2.jar"
        ),
        32 << 20,
    )?;
    let tiny = remap::tiny_from_jar(&jar)?;
    let manifest: serde_json::Value = serde_json::from_slice(&get(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        8 << 20,
    )?)
    .map_err(|e| e.to_string())?;
    let url = manifest["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|v| v["id"] == version)
        .and_then(|v| v["url"].as_str())
        .ok_or_else(|| format!("Mojang has no Minecraft {version}"))?;
    let info: serde_json::Value =
        serde_json::from_slice(&get(url, 8 << 20)?).map_err(|e| e.to_string())?;
    let mappings_url = info["downloads"]["client_mappings"]["url"]
        .as_str()
        .ok_or_else(|| format!("Minecraft {version} has no mappings"))?;
    let proguard = String::from_utf8(get(mappings_url, 64 << 20)?).map_err(|e| e.to_string())?;
    let m = Mappings::join(&tiny, &proguard)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let _ = fs::write(&kept, m.save());
    Ok(m)
}

/// The mappings for one Minecraft version, downloaded the first time a jar needs them.
pub struct Lazy<'a> {
    game: &'a Path,
    version: String,
    got: Option<Result<Mappings, String>>,
}

impl<'a> Lazy<'a> {
    pub fn new(game: &'a Path, version: &str) -> Self {
        Lazy {
            game,
            version: minecraft_version(version),
            got: None,
        }
    }

    fn get(&mut self) -> Result<&Mappings, String> {
        if self.got.is_none() {
            self.got = Some(mappings(self.game, &self.version));
        }
        match self.got.as_ref() {
            Some(Ok(m)) => Ok(m),
            Some(Err(e)) => Err(e.clone()),
            None => Err("no mappings".into()),
        }
    }
}

/// A mod made ready for lane 3: its DEX and the classes it starts with.
#[derive(Clone, Debug)]
pub struct Prepared {
    pub id: String,
    /// The jar file it came from (the report's name for it).
    pub file: String,
    pub entrypoints: Vec<String>,
    pub mixins: usize,
}

/// What lane 3 did with one jar.
#[derive(Clone, Debug)]
pub enum Outcome {
    /// Ready (and, after `check`, started).
    Ready(Prepared),
    /// Not for lane 3: why.
    Skipped(String),
    Failed(String),
}

/// The Fabric API's own modules and the loader: Horizon's API slice stands for them.
fn shimmed(id: &str) -> bool {
    id == "fabricloader" || id == "fabric-api" || id.starts_with("fabric-") || id == "minecraft"
}

/// Makes the jar `jar` (its file name `file`) ready for lane 3 in `lane3/<id>/`.
pub fn prepare(jar: &Path, file: &str, game: &Path, mappings: &mut Lazy) -> Outcome {
    let dir = game.join(DIR);
    let scratch = dir.join(".work");
    let _ = fs::remove_dir_all(&scratch);
    if let Err(e) = fs::create_dir_all(&scratch) {
        return Outcome::Failed(e.to_string());
    }
    let result = prepare_in(jar, file, &dir, &scratch, mappings);
    let _ = fs::remove_dir_all(&scratch);
    result
}

fn prepare_in(jar: &Path, file: &str, dir: &Path, scratch: &Path, mappings: &mut Lazy) -> Outcome {
    // First a look at what it is, without renaming.
    let first = scratch.join("look.jar");
    let look = match remap::remap_jar(jar, &first, None) {
        Ok(u) => u,
        Err(e) => return Outcome::Failed(e),
    };
    if look.fabric.is_none()
        && let Some(neo) = look.neoforge.clone()
    {
        return prepare_neoforge(&first, file, dir, &neo, look.classes);
    }
    let Some(fabric) = look.fabric.clone() else {
        return Outcome::Skipped(
            "lane 3 runs Fabric and NeoForge mods (Forge's API is not in it yet)".into(),
        );
    };
    if look.classes == 0 {
        return Outcome::Skipped("no Java code".into());
    }
    if shimmed(&fabric.id) {
        return Outcome::Skipped("part of Fabric itself: Horizon's API slice stands for it".into());
    }
    let id: String = fabric
        .id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if id.is_empty() {
        return Outcome::Failed("no mod id".into());
    }
    let m = match mappings.get() {
        Ok(m) => m,
        Err(e) => return Outcome::Failed(format!("mappings not downloaded: {e}")),
    };
    let mut inputs = Vec::new();
    let mut entrypoints: Vec<String> = fabric
        .entrypoints
        .iter()
        .filter(|(kind, _)| kind == "main")
        .map(|(_, class)| class.clone())
        .collect();
    let mut mixins = fabric.mixins;
    // The mod, then the libraries it carries (jar in jar), each renamed; Fabric's own modules
    // are left out, as the loader would find them in Fabric API.
    let mut queue = vec![(jar.to_path_buf(), 0usize)];
    let mut n = 0;
    while let Some((path, depth)) = queue.pop() {
        n += 1;
        let out = scratch.join(format!("{n}.jar"));
        let unpacked = match remap::remap_jar(&path, &out, Some(m)) {
            Ok(u) => u,
            Err(e) if n == 1 => return Outcome::Failed(e),
            Err(_) => continue,
        };
        if n > 1 {
            if let Some(lib) = &unpacked.fabric {
                if shimmed(&lib.id) {
                    continue;
                }
                entrypoints.extend(
                    lib.entrypoints
                        .iter()
                        .filter(|(kind, _)| kind == "main")
                        .map(|(_, class)| class.clone()),
                );
                mixins += lib.mixins;
            }
        }
        if unpacked.classes > 0 {
            inputs.push(out);
        }
        if depth < 3 {
            for (name, bytes) in unpacked.nested {
                n += 1;
                let nested = scratch.join(format!(
                    "{n}-{}",
                    name.rsplit('/').next().unwrap_or("lib.jar")
                ));
                if fs::write(&nested, bytes).is_ok() {
                    queue.push((nested, depth + 1));
                }
            }
        }
    }
    let target = dir.join(&id);
    let part = dir.join(format!(".{id}.part"));
    let _ = fs::remove_dir_all(&part);
    if let Err(e) = fs::create_dir_all(&part) {
        return Outcome::Failed(e.to_string());
    }
    if let Err(e) = dex(&inputs, &part.join("classes.jar")) {
        let _ = fs::remove_dir_all(&part);
        return Outcome::Failed(format!("D8 could not make DEX: {e}"));
    }
    let entry = format!(
        "file\t{file}\nmixins\t{mixins}\n{}",
        entrypoints
            .iter()
            .map(|c| format!("main\t{c}\n"))
            .collect::<String>()
    );
    if let Err(e) = fs::write(part.join("entry.txt"), entry) {
        return Outcome::Failed(e.to_string());
    }
    let _ = fs::remove_dir_all(&target);
    if let Err(e) = fs::rename(&part, &target) {
        return Outcome::Failed(e.to_string());
    }
    Outcome::Ready(Prepared {
        id,
        file: file.to_owned(),
        entrypoints,
        mixins,
    })
}

/// A NeoForge mod (already in Mojang's names, `jar` its copy without signatures) made ready
/// for lane 3: its `@Mod` classes start it, its `@EventBusSubscriber` classes hear events.
fn prepare_neoforge(
    jar: &Path,
    file: &str,
    dir: &Path,
    neo: &remap::NeoForgeMod,
    classes: usize,
) -> Outcome {
    if classes == 0 {
        return Outcome::Skipped("no Java code".into());
    }
    if matches!(neo.id.as_str(), "neoforge" | "minecraft") {
        return Outcome::Skipped(
            "part of NeoForge itself: Horizon's API slice stands for it".into(),
        );
    }
    let id: String = neo
        .id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if id.is_empty() {
        return Outcome::Failed("no mod id".into());
    }
    let target = dir.join(&id);
    let part = dir.join(format!(".{id}.part"));
    let _ = fs::remove_dir_all(&part);
    if let Err(e) = fs::create_dir_all(&part) {
        return Outcome::Failed(e.to_string());
    }
    if let Err(e) = dex(&[jar.to_path_buf()], &part.join("classes.jar")) {
        let _ = fs::remove_dir_all(&part);
        return Outcome::Failed(format!("D8 could not make DEX: {e}"));
    }
    let mut entry = format!("file\t{file}\nloader\tneoforge\n");
    for class in &neo.mods {
        entry.push_str(&format!("neoforge\t{class}\n"));
    }
    for class in &neo.subscribers {
        entry.push_str(&format!("subscriber\t{class}\n"));
    }
    if let Err(e) = fs::write(part.join("entry.txt"), entry) {
        return Outcome::Failed(e.to_string());
    }
    let _ = fs::remove_dir_all(&target);
    if let Err(e) = fs::rename(&part, &target) {
        return Outcome::Failed(e.to_string());
    }
    Outcome::Ready(Prepared {
        id,
        file: file.to_owned(),
        entrypoints: neo.mods.clone(),
        mixins: 0,
    })
}

/// The mods kept in `lane3/` that `on` accepts, in the order given by `order` (libraries
/// first), as `start` takes them.
pub fn installed(game: &Path, on: &dyn Fn(&str) -> bool) -> (Vec<PathBuf>, Vec<(String, String)>) {
    let mut dirs: Vec<PathBuf> = fs::read_dir(game.join(DIR))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("entry.txt").is_file() && p.join("classes.jar").is_file())
        .collect();
    dirs.sort();
    let mut dex = Vec::new();
    let mut entries = Vec::new();
    for d in dirs {
        let Some(id) = d.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        if id.starts_with('.') || !on(&id) {
            continue;
        }
        dex.push(d.join("classes.jar"));
        for line in fs::read_to_string(d.join("entry.txt"))
            .unwrap_or_default()
            .lines()
        {
            if let Some(class) = line.strip_prefix("main\t") {
                entries.push((id.clone(), class.to_owned()));
            } else if let Some(class) = line.strip_prefix("neoforge\t") {
                // NeoForge's: Lane3.start constructs them with the mod's event bus.
                entries.push((id.clone(), format!("neoforge:{class}")));
            } else if let Some(class) = line.strip_prefix("subscriber\t") {
                entries.push((id.clone(), format!("subscriber:{class}")));
            }
        }
    }
    (dex, entries)
}

/// Checks prepared mods by starting them once (outside a game), with every lane 3 mod already
/// kept: the result per mod id.
pub fn check(game: &Path, prepared: &[Prepared]) -> Result<Vec<(String, Started)>, String> {
    if crate::document_store::game_active() {
        return Err("a game is running: they are checked when a world opens".into());
    }
    let ids: Vec<&str> = prepared.iter().map(|p| p.id.as_str()).collect();
    let others = horizon_client::mod_list::ModList::load(game);
    let (dex, entries) = installed(game, &|id: &str| {
        ids.contains(&id) || others.enabled.contains(id)
    });
    let result = start(&dex, &entries, &game.join(DIR));
    stop();
    let mut result = result?;
    // A mod with no entrypoint of its own (a library) started fine if nothing threw.
    for p in prepared {
        if !result.iter().any(|(id, _)| *id == p.id) {
            result.push((
                p.id.clone(),
                Started::Ran("no entrypoint (a library)".into()),
            ));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_minecraft_version_a_mod_asks_for() {
        assert_eq!(minecraft_version("~1.21.1"), "1.21.1");
        assert_eq!(minecraft_version(">=1.21-"), "1.21");
        assert_eq!(minecraft_version(""), "1.21.1");
    }
}
