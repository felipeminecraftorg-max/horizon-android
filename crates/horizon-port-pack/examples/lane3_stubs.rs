//! Lane 3's stand-ins for one mod, on the desktop: the API map from the official client jar
//! (named by Mojang's client mappings) and the loader's jars, the mod's references, and
//! Horizon's compiled API slice; writes the stand-in classes to a folder for a JVM to load
//! beside the mod (the CI check in `.github/workflows/android-converter-tests.yml`).
//!
//! `cargo run -p horizon-port-pack --example lane3_stubs -- <client.jar> <client.txt>
//! <slice classes dir> <out dir> <mod.jar> [loader jars...]`
use horizon_port_pack::api::{Api, ApiClass, read};
use horizon_port_pack::stubs::{Known, Provided, Refs, stubs};
use std::{collections::HashMap, fs, io::Read, path::Path};

fn classes_of(jar: &Path, refs: &mut Refs, depth: usize) -> Result<usize, String> {
    let file = fs::File::open(jar).map_err(|e| format!("{}: {e}", jar.display()))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut n = 0;
    let mut nested = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().map_err(|e| e.to_string())?.into_owned();
        let mut bytes = Vec::new();
        if name.ends_with(".class") && !name.starts_with("META-INF/") {
            entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            if refs.add_class(&bytes).is_ok() {
                n += 1;
            }
        } else if name.ends_with(".jar") && name.starts_with("META-INF/jar") && depth < 3 {
            entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
            nested.push(bytes);
        }
    }
    for (i, bytes) in nested.into_iter().enumerate() {
        let path = std::env::temp_dir().join(format!("lane3-nested-{depth}-{i}.jar"));
        fs::write(&path, bytes).map_err(|e| e.to_string())?;
        n += classes_of(&path, refs, depth + 1)?;
    }
    Ok(n)
}

fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "class") {
            out.push(p);
        }
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [client, mappings, slice, out, jar, loaders @ ..] = args.as_slice() else {
        return Err(
            "usage: <client.jar> <client.txt> <slice dir> <out dir> <mod.jar> [loader jars]".into(),
        );
    };
    let mut api = Api::default();
    let proguard = fs::read_to_string(mappings).map_err(|e| e.to_string())?;
    let game = api.add_official(Path::new(client), &proguard)?;
    let mut loader = 0;
    for l in loaders {
        loader += api.add_jar(Path::new(l), None)?;
    }
    let mut refs = Refs::default();
    let read_classes = classes_of(Path::new(jar), &mut refs, 0)?;
    let mut files = Vec::new();
    walk(Path::new(slice), &mut files);
    let held: HashMap<String, ApiClass> = files
        .iter()
        .filter_map(|p| read(&fs::read(p).ok()?).ok())
        .collect();
    let class = |c: &str| {
        held.get(c).map(|k| Known {
            interface: k.is_interface(),
            plain: k
                .methods
                .get(&("<init>".to_owned(), "()V".to_owned()))
                .is_some_and(|a| a & 1 != 0),
        })
    };
    let constructor = |c: &str, d: &str| {
        held.get(c)
            .is_some_and(|k| k.methods.contains_key(&("<init>".to_owned(), d.to_owned())))
    };
    let member = |c: &str, n: &str, d: &str| {
        let mut at = Some(c.to_owned());
        let mut steps = 0;
        while let Some(name) = at.take() {
            let Some(k) = held.get(&name) else { break };
            if k.methods.contains_key(&(n.to_owned(), d.to_owned()))
                || k.fields.iter().any(|f| f.0 == n && f.1 == d)
            {
                return true;
            }
            steps += 1;
            if steps < 64 {
                at = Some(k.parent.clone());
            }
        }
        false
    };
    let ancestors = |c: &str| {
        let mut out = std::collections::BTreeSet::new();
        let mut queue = vec![c.to_owned()];
        while let Some(name) = queue.pop() {
            let Some(k) = held.get(&name) else { continue };
            for up in std::iter::once(&k.parent).chain(&k.interfaces) {
                if !up.is_empty() && out.insert(up.clone()) {
                    queue.push(up.clone());
                }
            }
        }
        out
    };
    let made = stubs(
        &refs,
        &Provided {
            class: &class,
            constructor: &constructor,
            member: &member,
            ancestors: &ancestors,
        },
        &api,
    );
    for (name, bytes) in &made.classes {
        let p = Path::new(out).join(format!("{name}.class"));
        fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
        fs::write(p, bytes).map_err(|e| e.to_string())?;
    }
    println!(
        "API: {game} game classes, {loader} loader classes. Mod: {read_classes} classes naming {}. \
         Stand-ins: {}. Slice gaps: {}.",
        refs.classes.len(),
        made.classes.len(),
        made.unmet.len()
    );
    for gap in made.unmet.iter().take(150) {
        println!("  gap: {gap}");
    }
    Ok(())
}
