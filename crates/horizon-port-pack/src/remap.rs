//! Lane 3 of the Android converter, the names: a Fabric mod's jar is compiled against Fabric's
//! intermediary names (`net/minecraft/class_1937`, `method_8320`), and Horizon's Java API shim
//! has Mojang's (`net/minecraft/world/level/Level`, `getBlockState`), as NeoForge mods do. The
//! mappings come from the player's own downloads: Fabric's intermediary (`official` to
//! `intermediary`, CC0) and Mojang's client mappings (Mojang's names to `official`); joined, they
//! rename intermediary to Mojang's.
//!
//! Renaming rewrites a class file's UTF-8 constants only: class names (in class entries,
//! descriptors and signatures), and members by their intermediary name, which is the same for a
//! method and every override of it. Nothing else of the class changes, so no offset moves.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::Path,
};

/// Intermediary names to Mojang's.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Mappings {
    /// Internal class names: `net/minecraft/class_1937` → `net/minecraft/world/level/Level`.
    pub classes: HashMap<String, String>,
    /// Members: `method_8320` → `getBlockState`, `field_9236` → `isClientSide`.
    pub members: HashMap<String, String>,
}

/// A Java type of Mojang's mappings (`int`, `net.minecraft.core.BlockPos[]`) as a descriptor,
/// classes named by `class` (Mojang's internal name → official).
fn descriptor(java: &str, class: &HashMap<String, String>) -> String {
    let mut ty = java.trim();
    let mut out = String::new();
    while let Some(inner) = ty.strip_suffix("[]") {
        out.push('[');
        ty = inner;
    }
    out.push_str(match ty {
        "boolean" => "Z",
        "byte" => "B",
        "char" => "C",
        "short" => "S",
        "int" => "I",
        "long" => "J",
        "float" => "F",
        "double" => "D",
        "void" => "V",
        _ => {
            let internal = ty.replace('.', "/");
            let name = class.get(&internal).cloned().unwrap_or(internal);
            out.push('L');
            out.push_str(&name);
            out.push(';');
            return out;
        }
    });
    out
}

/// What Mojang's client mappings (ProGuard's format) say, by official name.
#[derive(Default)]
pub struct Mojang {
    /// Official internal class name → Mojang's.
    pub(crate) classes: HashMap<String, String>,
    /// (official class, official field) → Mojang's field name.
    pub(crate) fields: HashMap<(String, String), String>,
    /// (official class, official name, official descriptor) → Mojang's method name.
    pub(crate) methods: HashMap<(String, String, String), String>,
}

pub(crate) fn mojang(proguard: &str) -> Result<Mojang, String> {
    // Classes first: a method's descriptor names classes of any part of the file.
    let mut named_to_official = HashMap::new();
    for line in proguard.lines() {
        if line.starts_with('#') || line.starts_with(' ') || line.trim().is_empty() {
            continue;
        }
        let (named, official) = line
            .trim_end_matches(':')
            .split_once(" -> ")
            .ok_or_else(|| format!("Mojang's mappings: unreadable line {line:?}"))?;
        named_to_official.insert(named.replace('.', "/"), official.replace('.', "/"));
    }
    let mut out = Mojang::default();
    for (named, official) in &named_to_official {
        out.classes.insert(official.clone(), named.clone());
    }
    let mut class: Option<String> = None;
    for line in proguard.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with(' ') {
            class = line
                .trim_end_matches(':')
                .split_once(" -> ")
                .map(|(_, official)| official.replace('.', "/"));
            continue;
        }
        let Some(owner) = &class else { continue };
        let Some((member, official)) = line.trim().split_once(" -> ") else {
            continue;
        };
        // `1:5:void <init>(int):12:13` and `int field`.
        let member = member.trim_start_matches(|c: char| c.is_ascii_digit() || c == ':');
        let Some((ty, rest)) = member.split_once(' ') else {
            continue;
        };
        match rest.split_once('(') {
            Some((name, args)) => {
                let args = args.split(')').next().unwrap_or("");
                let mut desc = String::from("(");
                for a in args.split(',').filter(|a| !a.trim().is_empty()) {
                    desc.push_str(&descriptor(a, &named_to_official));
                }
                desc.push(')');
                desc.push_str(&descriptor(ty, &named_to_official));
                out.methods
                    .insert((owner.clone(), official.to_owned(), desc), name.to_owned());
            }
            None => {
                out.fields
                    .insert((owner.clone(), official.to_owned()), rest.to_owned());
            }
        }
    }
    Ok(out)
}

impl Mappings {
    /// Joins Fabric's intermediary (`tiny` v2, `official` first) and Mojang's client mappings.
    pub fn join(tiny: &str, proguard: &str) -> Result<Self, String> {
        let mojang = mojang(proguard)?;
        let mut lines = tiny.lines();
        let header: Vec<&str> = lines
            .next()
            .ok_or("intermediary: empty file")?
            .split('\t')
            .collect();
        if header.first() != Some(&"tiny") || header.get(1) != Some(&"2") {
            return Err("intermediary: not a tiny v2 file".into());
        }
        let names = &header[3..];
        let official = names.iter().position(|n| *n == "official");
        let inter = names.iter().position(|n| *n == "intermediary");
        let (Some(0), Some(inter)) = (official, inter) else {
            return Err("intermediary: needs `official` first and `intermediary`".into());
        };
        let mut out = Mappings::default();
        let mut class: Option<String> = None;
        for line in lines {
            let parts: Vec<&str> = line.split('\t').collect();
            match parts.as_slice() {
                ["c", names @ ..] => {
                    let (Some(obf), Some(int)) = (names.first(), names.get(inter)) else {
                        continue;
                    };
                    if let Some(named) = mojang.classes.get(*obf) {
                        out.classes.insert((*int).to_owned(), named.clone());
                    }
                    class = Some((*obf).to_owned());
                }
                ["", "m", desc, names @ ..] => {
                    let (Some(owner), Some(obf), Some(int)) =
                        (&class, names.first(), names.get(inter))
                    else {
                        continue;
                    };
                    let key = (owner.clone(), (*obf).to_owned(), (*desc).to_owned());
                    if let Some(named) = mojang.methods.get(&key) {
                        out.members
                            .entry((*int).to_owned())
                            .or_insert_with(|| named.clone());
                    }
                }
                ["", "f", _desc, names @ ..] => {
                    let (Some(owner), Some(obf), Some(int)) =
                        (&class, names.first(), names.get(inter))
                    else {
                        continue;
                    };
                    let key = (owner.clone(), (*obf).to_owned());
                    if let Some(named) = mojang.fields.get(&key) {
                        out.members
                            .entry((*int).to_owned())
                            .or_insert_with(|| named.clone());
                    }
                }
                _ => {}
            }
        }
        if out.classes.is_empty() {
            return Err("the mappings name no class".into());
        }
        Ok(out)
    }

    /// One line per name: `c\t<intermediary>\t<Mojang's>`, `m\t...`.
    pub fn save(&self) -> String {
        let mut lines: Vec<String> = self
            .classes
            .iter()
            .map(|(i, n)| format!("c\t{i}\t{n}"))
            .chain(self.members.iter().map(|(i, n)| format!("m\t{i}\t{n}")))
            .collect();
        lines.sort();
        lines.join("\n")
    }

    pub fn load(text: &str) -> Self {
        let mut out = Mappings::default();
        for line in text.lines() {
            match line.split('\t').collect::<Vec<_>>().as_slice() {
                ["c", i, n] => {
                    out.classes.insert((*i).to_owned(), (*n).to_owned());
                }
                ["m", i, n] => {
                    out.members.insert((*i).to_owned(), (*n).to_owned());
                }
                _ => {}
            }
        }
        out
    }

    /// A UTF-8 constant renamed, or None when it names nothing intermediary.
    pub fn rename(&self, s: &str) -> Option<String> {
        if let Some((kind, number)) = s.split_once('_')
            && matches!(kind, "method" | "field" | "comp")
            && !number.is_empty()
            && number.bytes().all(|b| b.is_ascii_digit())
        {
            return self.members.get(s).cloned();
        }
        if !s.contains("class_") {
            return None;
        }
        let mut out = String::with_capacity(s.len());
        let mut changed = false;
        let mut rest = s;
        loop {
            let slash = rest.find("net/minecraft/class_");
            let dot = rest.find("net.minecraft.class_");
            let (at, dotted) = match (slash, dot) {
                (Some(a), Some(b)) if b < a => (b, true),
                (Some(a), _) => (a, false),
                (None, Some(b)) => (b, true),
                (None, None) => break,
            };
            out.push_str(&rest[..at]);
            let tail = &rest[at..];
            // `net/minecraft/class_N` then any `$class_N`.
            let mut end = "net/minecraft/class_".len();
            let digits = |from: usize| tail[from..].bytes().take_while(u8::is_ascii_digit).count();
            let n = digits(end);
            if n == 0 {
                out.push_str(&tail[..end]);
                rest = &tail[end..];
                continue;
            }
            end += n;
            while tail[end..].starts_with("$class_") && digits(end + 7) > 0 {
                end += 7 + digits(end + 7);
            }
            let name = tail[..end].replace('.', "/");
            match self.classes.get(&name) {
                Some(named) => {
                    changed = true;
                    if dotted {
                        out.push_str(&named.replace('/', "."));
                    } else {
                        out.push_str(named);
                    }
                }
                None => out.push_str(&tail[..end]),
            }
            rest = &tail[end..];
        }
        out.push_str(rest);
        changed.then_some(out)
    }
}

/// A class file with its UTF-8 constants renamed by `mappings`.
pub fn remap_class(bytes: &[u8], mappings: &Mappings) -> Result<Vec<u8>, String> {
    let u2 = |at: usize| -> Result<usize, String> {
        bytes
            .get(at..at + 2)
            .map(|b| usize::from(u16::from_be_bytes([b[0], b[1]])))
            .ok_or_else(|| "class file cut short".to_owned())
    };
    if bytes.get(..4) != Some(&[0xCA, 0xFE, 0xBA, 0xBE][..]) {
        return Err("not a class file".into());
    }
    let count = u2(8)?;
    let mut out = bytes[..10].to_vec();
    let mut at = 10;
    let mut index = 1;
    while index < count {
        let tag = *bytes.get(at).ok_or("class file cut short")?;
        let size = match tag {
            1 => {
                let len = u2(at + 1)?;
                let raw = bytes
                    .get(at + 3..at + 3 + len)
                    .ok_or("class file cut short")?;
                let renamed = std::str::from_utf8(raw)
                    .ok()
                    .and_then(|s| mappings.rename(s))
                    .filter(|s| s.len() <= usize::from(u16::MAX));
                match renamed {
                    Some(s) => {
                        out.push(1);
                        out.extend_from_slice(&(s.len() as u16).to_be_bytes());
                        out.extend_from_slice(s.as_bytes());
                    }
                    None => out.extend_from_slice(&bytes[at..at + 3 + len]),
                }
                at += 3 + len;
                index += 1;
                continue;
            }
            3 | 4 => 5,
            5 | 6 => {
                // Longs and doubles take two slots.
                index += 1;
                9
            }
            7 | 8 | 16 | 19 | 20 => 3,
            9 | 10 | 11 | 12 | 17 | 18 => 5,
            15 => 4,
            other => return Err(format!("unknown constant tag {other}")),
        };
        out.extend_from_slice(bytes.get(at..at + size).ok_or("class file cut short")?);
        at += size;
        index += 1;
    }
    out.extend_from_slice(&bytes[at..]);
    Ok(out)
}

/// A Fabric mod's own description (`fabric.mod.json`): its id and the classes it starts with.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct FabricMod {
    pub id: String,
    pub name: String,
    /// (`main`, `client` or `server`, class name).
    pub entrypoints: Vec<(String, String)>,
    pub mixins: usize,
    /// The Minecraft version it asks for (`~1.21.1`, `>=1.21`).
    pub minecraft: String,
}

impl FabricMod {
    pub fn parse(json: &str) -> Result<Self, String> {
        let v: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let text = |key: &str| v[key].as_str().unwrap_or("").to_owned();
        let mut entrypoints = Vec::new();
        for (kind, list) in v["entrypoints"].as_object().into_iter().flatten() {
            for e in list.as_array().into_iter().flatten() {
                let class = e.as_str().or_else(|| e["value"].as_str());
                if let Some(class) = class {
                    entrypoints.push((kind.clone(), class.to_owned()));
                }
            }
        }
        let minecraft = match &v["depends"]["minecraft"] {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(a) => a.first().and_then(|s| s.as_str()).unwrap_or("").into(),
            _ => String::new(),
        };
        Ok(FabricMod {
            id: text("id"),
            name: text("name"),
            entrypoints,
            mixins: v["mixins"].as_array().map_or(0, Vec::len),
            minecraft,
        })
    }
}

/// Fabric's intermediary mappings (`mappings/mappings.tiny`) from their jar's bytes.
pub fn tiny_from_jar(bytes: &[u8]) -> Result<String, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| format!("intermediary: {e}"))?;
    let mut tiny = String::new();
    zip.by_name("mappings/mappings.tiny")
        .map_err(|e| format!("intermediary: {e}"))?
        .read_to_string(&mut tiny)
        .map_err(|e| format!("intermediary: {e}"))?;
    Ok(tiny)
}

/// What a jar holds for lane 3.
#[derive(Debug, Default)]
pub struct Unpacked {
    pub fabric: Option<FabricMod>,
    /// A NeoForge mod's own description and the classes the loader starts.
    pub neoforge: Option<NeoForgeMod>,
    /// The jars it carries in `META-INF/jars/` (libraries), as bytes.
    pub nested: Vec<(String, Vec<u8>)>,
    pub classes: usize,
}

/// A NeoForge mod (`META-INF/neoforge.mods.toml`): its id, its `@Mod` classes (the loader
/// constructs them with the mod's event bus) and its `@EventBusSubscriber` classes (whose
/// `@SubscribeEvent` static methods hear the game's events). NeoForge mods have Mojang's names.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct NeoForgeMod {
    pub id: String,
    pub mods: Vec<String>,
    pub subscribers: Vec<String>,
}

impl NeoForgeMod {
    /// The first `modId = "..."` of a `neoforge.mods.toml`.
    pub fn id_of(toml: &str) -> Option<String> {
        toml.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == "modId").then(|| v.trim().trim_matches('"').to_owned())
        })
    }
    /// Notes the class `name` (`a/b/C.class`) by the annotations its constant pool names.
    pub fn add_class(&mut self, name: &str, bytes: &[u8]) {
        let class = name.trim_end_matches(".class").replace('/', ".");
        let names = |needle: &[u8]| bytes.windows(needle.len()).any(|w| w == needle);
        if names(b"Lnet/neoforged/fml/common/Mod;") {
            self.mods.push(class.clone());
        }
        if names(b"Lnet/neoforged/fml/common/EventBusSubscriber;")
            || names(b"Lnet/neoforged/fml/common/Mod$EventBusSubscriber;")
        {
            self.subscribers.push(class);
        }
    }
}

/// No class or nested jar is this large; a bigger one is left out.
const MAX_ENTRY: u64 = 64 << 20;

/// Copies the classes and resources of the jar at `input` into the jar at `output`, its classes
/// renamed by `mappings` (None for a jar already in Mojang's names), its signatures and nested
/// jars left out. Returns what it found.
pub fn remap_jar(
    input: &Path,
    output: &Path,
    mappings: Option<&Mappings>,
) -> Result<Unpacked, String> {
    let file = fs::File::open(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", input.display()))?;
    if let Some(dir) = output.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let out = fs::File::create(output).map_err(|e| format!("{}: {e}", output.display()))?;
    let mut writer = zip::ZipWriter::new(out);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let mut found = Unpacked::default();
    // The classes' NeoForge annotations, kept until the jar's mods.toml says it is one.
    let mut annotated = NeoForgeMod::default();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().map_err(|e| e.to_string())?.into_owned();
        if entry.is_dir() || entry.size() > MAX_ENTRY {
            continue;
        }
        let upper = name.to_ascii_uppercase();
        if upper.starts_with("META-INF/")
            && [".SF", ".RSA", ".DSA", ".EC"]
                .iter()
                .any(|s| upper.ends_with(s))
        {
            continue;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| format!("{name}: {e}"))?;
        if name.starts_with("META-INF/jars/") && name.ends_with(".jar") {
            found.nested.push((name, bytes));
            continue;
        }
        if name == "fabric.mod.json" {
            found.fabric = FabricMod::parse(&String::from_utf8_lossy(&bytes)).ok();
        }
        if name == "META-INF/neoforge.mods.toml"
            && let Some(id) = NeoForgeMod::id_of(&String::from_utf8_lossy(&bytes))
        {
            found.neoforge.get_or_insert_with(NeoForgeMod::default).id = id;
        }
        if name.ends_with(".class") {
            found.classes += 1;
            if mappings.is_none() {
                annotated.add_class(&name, &bytes);
            }
            if let Some(m) = mappings {
                bytes = remap_class(&bytes, m).map_err(|e| format!("{name}: {e}"))?;
            }
        }
        writer
            .start_file(name.as_str(), options)
            .map_err(|e| format!("{name}: {e}"))?;
        writer
            .write_all(&bytes)
            .map_err(|e| format!("{name}: {e}"))?;
    }
    writer.finish().map_err(|e| e.to_string())?;
    if let Some(n) = found.neoforge.as_mut() {
        n.mods = annotated.mods;
        n.subscribers = annotated.subscribers;
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY: &str = "tiny\t2\t0\tofficial\tintermediary\n\
c\tdtc\tnet/minecraft/class_2680\n\
c\tdcw\tnet/minecraft/class_1937\n\
\tm\t(Lji;)Ldtc;\ta_\tmethod_8320\n\
\tf\tZ\ty\tfield_9236\n\
c\tji\tnet/minecraft/class_2338\n\
c\tji$a\tnet/minecraft/class_2338$class_2339\n";

    // Members are indented, so no line continuation (it would eat the indent).
    const MOJANG: &str = concat!(
        "# comment\n",
        "net.minecraft.world.level.block.state.BlockState -> dtc:\n",
        "net.minecraft.world.level.Level -> dcw:\n",
        "    boolean isClientSide -> y\n",
        "    12:12:net.minecraft.world.level.block.state.BlockState ",
        "getBlockState(net.minecraft.core.BlockPos):34:34 -> a_\n",
        "net.minecraft.core.BlockPos -> ji:\n",
        "net.minecraft.core.BlockPos$MutableBlockPos -> ji$a:\n",
    );

    #[test]
    fn a_neoforge_mod_is_known_by_its_toml_and_its_annotations() {
        assert_eq!(
            NeoForgeMod::id_of("modLoader=\"javafml\"\n[[mods]]\nmodId = \"places\"\n").as_deref(),
            Some("places")
        );
        let mut n = NeoForgeMod::default();
        n.add_class("a/Main.class", b"..Lnet/neoforged/fml/common/Mod;..");
        n.add_class(
            "a/Events.class",
            b"..Lnet/neoforged/fml/common/EventBusSubscriber;..",
        );
        n.add_class("a/Other.class", b"..Lnet/neoforged/fml/common/ModList;..");
        assert_eq!(n.mods, ["a.Main"]);
        assert_eq!(n.subscribers, ["a.Events"]);
    }

    #[test]
    fn intermediary_names_become_mojang_s() {
        let m = Mappings::join(TINY, MOJANG).unwrap();
        assert_eq!(
            m.classes["net/minecraft/class_1937"],
            "net/minecraft/world/level/Level"
        );
        assert_eq!(m.members["method_8320"], "getBlockState");
        assert_eq!(m.members["field_9236"], "isClientSide");
        assert_eq!(
            m.rename("(Lnet/minecraft/class_2338;)Lnet/minecraft/class_2680;")
                .as_deref(),
            Some(
                "(Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/level/block/state/BlockState;"
            )
        );
        assert_eq!(
            m.rename("Lnet/minecraft/class_2338$class_2339;").as_deref(),
            Some("Lnet/minecraft/core/BlockPos$MutableBlockPos;")
        );
        assert_eq!(
            m.rename("net.minecraft.class_1937").as_deref(),
            Some("net.minecraft.world.level.Level")
        );
        assert_eq!(m.rename("method_8320").as_deref(), Some("getBlockState"));
        assert_eq!(m.rename("hello"), None);
        assert_eq!(m.rename("net/minecraft/class_99999"), None);
        assert_eq!(Mappings::load(&m.save()), m);
    }

    #[test]
    fn a_class_file_s_constants_are_renamed_and_nothing_else_moves() {
        let m = Mappings::join(TINY, MOJANG).unwrap();
        // A pool of: Utf8 class name, Class, Long (two slots), Utf8 member, Utf8 other.
        let mut class = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 65];
        let utf8 = |s: &str| {
            let mut v = vec![1];
            v.extend_from_slice(&(s.len() as u16).to_be_bytes());
            v.extend_from_slice(s.as_bytes());
            v
        };
        let mut pool = Vec::new();
        pool.extend(utf8("net/minecraft/class_1937"));
        pool.extend([7, 0, 1]);
        pool.extend([5, 0, 0, 0, 0, 0, 0, 0, 42]);
        pool.extend(utf8("method_8320"));
        pool.extend(utf8("keep me"));
        class.extend_from_slice(&7u16.to_be_bytes());
        class.extend(pool);
        class.extend([0xAB, 0xCD]);
        let out = remap_class(&class, &m).unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("net/minecraft/world/level/Level"));
        assert!(text.contains("getBlockState") && text.contains("keep me"));
        assert!(!text.contains("class_1937") && !text.contains("method_8320"));
        assert!(out.ends_with(&[0xAB, 0xCD]));
        assert!(remap_class(b"nope", &m).is_err());
    }

    #[test]
    fn a_fabric_mod_s_entrypoints_are_read() {
        let m = FabricMod::parse(
            r#"{"id": "harvest", "name": "Harvest", "entrypoints": {"main": ["a.Main",
            {"adapter": "kotlin", "value": "a.Kt"}], "client": ["a.Client"]},
            "mixins": ["a.mixins.json"], "depends": {"minecraft": "~1.21.1"}}"#,
        )
        .unwrap();
        assert_eq!(m.id, "harvest");
        assert_eq!(m.entrypoints.len(), 3);
        assert!(m.entrypoints.contains(&("main".into(), "a.Kt".into())));
        assert_eq!((m.mixins, m.minecraft.as_str()), (1, "~1.21.1"));
    }
}
