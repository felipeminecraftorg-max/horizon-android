//! Lane 3's map of the Java API mods are written against: every class of the game, read from
//! the player's own download of the official client jar and named by Mojang's mappings, and
//! of the loader (NeoForge's jar), with what it extends and implements, whether it is an
//! interface or an enum, and its members' names, descriptors and access. Only that structure
//! is kept, never code. The stand-ins ([`crate::stubs`]) take their hierarchy from it, so a
//! mod's `MyBlock extends SlabBlock` still reaches `Block`, and a `ServerLevel` is a `Level`.
use std::collections::{BTreeMap, HashMap};

/// One class of the API.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ApiClass {
    /// Its superclass (empty for `java/lang/Object` itself and interfaces' `Object`).
    pub parent: String,
    pub interfaces: Vec<String>,
    pub access: u16,
    /// Public and protected methods: (name, descriptor) → access.
    pub methods: BTreeMap<(String, String), u16>,
    /// Public and protected fields, in declaration order: (name, descriptor, access).
    pub fields: Vec<(String, String, u16)>,
}

impl ApiClass {
    pub fn is_interface(&self) -> bool {
        self.access & 0x0200 != 0
    }
    pub fn is_enum(&self) -> bool {
        self.access & 0x4000 != 0
    }
    /// An enum's constants, in declaration order.
    pub fn constants(&self) -> impl Iterator<Item = &str> {
        self.fields
            .iter()
            .filter(|f| f.2 & 0x4000 != 0)
            .map(|f| f.0.as_str())
    }
}

/// The API: classes by internal name.
#[derive(Debug, Default, Clone)]
pub struct Api {
    pub classes: HashMap<String, ApiClass>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn u1(&mut self) -> Result<u8, String> {
        let b = *self.bytes.get(self.at).ok_or("class file cut short")?;
        self.at += 1;
        Ok(b)
    }
    fn u2(&mut self) -> Result<u16, String> {
        Ok(u16::from(self.u1()?) << 8 | u16::from(self.u1()?))
    }
    fn u4(&mut self) -> Result<u32, String> {
        Ok(u32::from(self.u2()?) << 16 | u32::from(self.u2()?))
    }
    fn skip(&mut self, n: usize) -> Result<(), String> {
        if self.at + n > self.bytes.len() {
            return Err("class file cut short".into());
        }
        self.at += n;
        Ok(())
    }
}

/// A class file's structure: its name and what [`ApiClass`] keeps.
pub fn read(bytes: &[u8]) -> Result<(String, ApiClass), String> {
    let mut r = Reader { bytes, at: 0 };
    if r.u4()? != 0xCAFE_BABE {
        return Err("not a class file".into());
    }
    r.skip(4)?;
    let count = r.u2()?;
    let mut utf8: HashMap<u16, String> = HashMap::new();
    let mut classes: HashMap<u16, u16> = HashMap::new();
    let mut i = 1;
    while i < count {
        match r.u1()? {
            1 => {
                let n = usize::from(r.u2()?);
                let s = bytes.get(r.at..r.at + n).ok_or("class file cut short")?;
                utf8.insert(i, String::from_utf8_lossy(s).into_owned());
                r.skip(n)?;
            }
            7 => {
                classes.insert(i, r.u2()?);
            }
            8 | 16 | 19 | 20 => r.skip(2)?,
            15 => r.skip(3)?,
            3 | 4 | 9..=12 | 17 | 18 => r.skip(4)?,
            5 | 6 => {
                r.skip(8)?;
                i += 1;
            }
            other => return Err(format!("unknown constant tag {other}")),
        }
        i += 1;
    }
    let text = |k: u16| utf8.get(&k).cloned().unwrap_or_default();
    let class = |k: u16| classes.get(&k).map(|n| text(*n)).unwrap_or_default();
    let access = r.u2()?;
    let name = class(r.u2()?);
    let parent = class(r.u2()?);
    let mut interfaces = Vec::new();
    for _ in 0..r.u2()? {
        interfaces.push(class(r.u2()?));
    }
    let mut out = ApiClass {
        parent,
        interfaces,
        access,
        ..ApiClass::default()
    };
    let visible = |a: u16| a & 0x0002 == 0 && a & 0x1000 == 0 && (a & 0x0005 != 0);
    let skip_attributes = |r: &mut Reader| -> Result<(), String> {
        for _ in 0..r.u2()? {
            r.skip(2)?;
            let n = r.u4()? as usize;
            r.skip(n)?;
        }
        Ok(())
    };
    for _ in 0..r.u2()? {
        let (a, n, d) = (r.u2()?, r.u2()?, r.u2()?);
        if visible(a) {
            out.fields.push((text(n), text(d), a));
        }
        skip_attributes(&mut r)?;
    }
    for _ in 0..r.u2()? {
        let (a, n, d) = (r.u2()?, r.u2()?, r.u2()?);
        if visible(a) {
            out.methods.insert((text(n), text(d)), a);
        }
        skip_attributes(&mut r)?;
    }
    Ok((name, out))
}

/// A descriptor or signature with its class names renamed by `classes`.
pub fn rename_descriptor(descriptor: &str, classes: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(descriptor.len());
    let mut rest = descriptor;
    while let Some(i) = rest.find('L') {
        out.push_str(&rest[..=i]);
        let after = &rest[i + 1..];
        let Some(end) = after.find(';') else {
            out.push_str(after);
            return out;
        };
        let name = &after[..end];
        out.push_str(classes.get(name).map_or(name, String::as_str));
        out.push(';');
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

impl Api {
    /// Adds a class file. Classes named by Mojang's mappings (official jar) are renamed.
    pub fn add_class(
        &mut self,
        bytes: &[u8],
        names: Option<&crate::remap::Mojang>,
    ) -> Result<(), String> {
        let (name, mut class) = read(bytes)?;
        let Some(m) = names else {
            self.classes.insert(name, class);
            return Ok(());
        };
        let rename = |c: &str| m.classes.get(c).cloned().unwrap_or_else(|| c.to_owned());
        let methods = std::mem::take(&mut class.methods);
        for ((method, descriptor), access) in methods {
            let named = m
                .methods
                .get(&(name.clone(), method.clone(), descriptor.clone()))
                .cloned()
                .unwrap_or(method);
            class
                .methods
                .insert((named, rename_descriptor(&descriptor, &m.classes)), access);
        }
        for field in &mut class.fields {
            if let Some(named) = m.fields.get(&(name.clone(), field.0.clone())) {
                field.0 = named.clone();
            }
            field.1 = rename_descriptor(&field.1, &m.classes);
        }
        class.parent = rename(&class.parent);
        for i in &mut class.interfaces {
            *i = rename(i);
        }
        self.classes.insert(rename(&name), class);
        Ok(())
    }

    /// Every class file of the jar at `path` (`names`: Mojang's mappings, for the official jar).
    pub fn add_jar(
        &mut self,
        path: &std::path::Path,
        names: Option<&crate::remap::Mojang>,
    ) -> Result<usize, String> {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut n = 0;
        for i in 0..zip.len() {
            let mut entry = zip.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().map_err(|e| e.to_string())?.into_owned();
            if !name.ends_with(".class") || name.starts_with("META-INF/") {
                continue;
            }
            let mut bytes = Vec::new();
            entry
                .read_to_end(&mut bytes)
                .map_err(|e| format!("{name}: {e}"))?;
            if self.add_class(&bytes, names).is_ok() {
                n += 1;
            }
        }
        Ok(n)
    }

    /// The official client jar, named by Mojang's client mappings (ProGuard's text).
    pub fn add_official(&mut self, jar: &std::path::Path, proguard: &str) -> Result<usize, String> {
        let names = crate::remap::mojang(proguard)?;
        self.add_jar(jar, Some(&names))
    }

    /// One line per class (`c\tname\tparent\taccess\tinterfaces`), method (`m\tname\tdescriptor\taccess`)
    /// and field (`f\tname\tdescriptor\taccess`), members under their class.
    pub fn save(&self) -> String {
        let mut names: Vec<&String> = self.classes.keys().collect();
        names.sort();
        let mut out = String::new();
        for name in names {
            let c = &self.classes[name];
            out.push_str(&format!(
                "c\t{name}\t{}\t{}\t{}\n",
                c.parent,
                c.access,
                c.interfaces.join(",")
            ));
            for (n, d, a) in &c.fields {
                out.push_str(&format!("f\t{n}\t{d}\t{a}\n"));
            }
            for ((n, d), a) in &c.methods {
                out.push_str(&format!("m\t{n}\t{d}\t{a}\n"));
            }
        }
        out
    }

    pub fn load(text: &str) -> Self {
        let mut api = Api::default();
        let mut current: Option<String> = None;
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            match parts.as_slice() {
                ["c", name, parent, access, interfaces] => {
                    api.classes.insert(
                        (*name).to_owned(),
                        ApiClass {
                            parent: (*parent).to_owned(),
                            interfaces: interfaces
                                .split(',')
                                .filter(|s| !s.is_empty())
                                .map(str::to_owned)
                                .collect(),
                            access: access.parse().unwrap_or(0),
                            ..ApiClass::default()
                        },
                    );
                    current = Some((*name).to_owned());
                }
                ["f", n, d, a] => {
                    if let Some(c) = current.as_ref().and_then(|c| api.classes.get_mut(c)) {
                        c.fields
                            .push(((*n).to_owned(), (*d).to_owned(), a.parse().unwrap_or(0)));
                    }
                }
                ["m", n, d, a] => {
                    if let Some(c) = current.as_ref().and_then(|c| api.classes.get_mut(c)) {
                        c.methods
                            .insert(((*n).to_owned(), (*d).to_owned()), a.parse().unwrap_or(0));
                    }
                }
                _ => {}
            }
        }
        api
    }

    /// `class`'s superclasses, nearest first, as far as the API knows them.
    pub fn parents(&self, class: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut at = class;
        while let Some(c) = self.classes.get(at) {
            if c.parent.is_empty() || out.len() > 64 {
                break;
            }
            out.push(c.parent.as_str());
            at = c.parent.as_str();
        }
        out
    }

    /// The class declaring method `name descriptor` as `class` resolves it: its superclasses
    /// first, then its interfaces (JVMS 5.4.3.3), with that declaration's access.
    pub fn declaring_method(
        &self,
        class: &str,
        name: &str,
        descriptor: &str,
    ) -> Option<(&str, u16)> {
        let key = (name.to_owned(), descriptor.to_owned());
        let mut chain = vec![class];
        chain.extend(self.parents(class));
        for c in &chain {
            if let Some((owner, info)) = self.classes.get_key_value(*c)
                && let Some(a) = info.methods.get(&key)
            {
                return Some((owner.as_str(), *a));
            }
        }
        let mut queue: Vec<&str> = chain
            .iter()
            .filter_map(|c| self.classes.get(*c))
            .flat_map(|c| c.interfaces.iter().map(String::as_str))
            .collect();
        let mut seen = std::collections::HashSet::new();
        while let Some(i) = queue.pop() {
            if !seen.insert(i) {
                continue;
            }
            if let Some((owner, info)) = self.classes.get_key_value(i) {
                if let Some(a) = info.methods.get(&key) {
                    return Some((owner.as_str(), *a));
                }
                queue.extend(info.interfaces.iter().map(String::as_str));
            }
        }
        None
    }

    /// The class declaring field `name descriptor` as `class` resolves it.
    pub fn declaring_field(
        &self,
        class: &str,
        name: &str,
        descriptor: &str,
    ) -> Option<(&str, u16)> {
        let mut chain = vec![class];
        chain.extend(self.parents(class));
        for c in chain {
            if let Some((owner, info)) = self.classes.get_key_value(c) {
                if let Some(f) = info
                    .fields
                    .iter()
                    .find(|f| f.0 == name && f.1 == descriptor)
                {
                    return Some((owner.as_str(), f.2));
                }
                for i in &info.interfaces {
                    if let Some(found) = self.declaring_field(i, name, descriptor) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn api() -> Api {
        let mut api = Api::default();
        let mut class = |name: &str, parent: &str, access: u16, interfaces: &[&str]| {
            api.classes.insert(
                name.to_owned(),
                ApiClass {
                    parent: parent.to_owned(),
                    interfaces: interfaces.iter().map(|s| (*s).to_owned()).collect(),
                    access,
                    ..ApiClass::default()
                },
            );
        };
        class("g/Behaviour", "java/lang/Object", 0x0421, &[]);
        class("g/Block", "g/Behaviour", 0x0021, &["g/ItemLike"]);
        class("g/Slab", "g/Block", 0x0021, &[]);
        class("g/ItemLike", "java/lang/Object", 0x0601, &[]);
        api.classes
            .get_mut("g/Behaviour")
            .unwrap()
            .methods
            .insert(("codec".into(), "()Lg/Codec;".into()), 0x0009);
        api.classes
            .get_mut("g/ItemLike")
            .unwrap()
            .methods
            .insert(("asItem".into(), "()Lg/Item;".into()), 0x0401);
        api
    }

    #[test]
    fn members_resolve_through_superclasses_then_interfaces() {
        let api = api();
        assert_eq!(
            api.parents("g/Slab"),
            ["g/Block", "g/Behaviour", "java/lang/Object"]
        );
        assert_eq!(
            api.declaring_method("g/Slab", "codec", "()Lg/Codec;"),
            Some(("g/Behaviour", 0x0009))
        );
        assert_eq!(
            api.declaring_method("g/Slab", "asItem", "()Lg/Item;"),
            Some(("g/ItemLike", 0x0401))
        );
        assert_eq!(api.declaring_method("g/Slab", "nothing", "()V"), None);
    }

    #[test]
    fn the_map_reads_back_as_it_was_saved() {
        let mut api = api();
        api.classes.get_mut("g/Block").unwrap().fields.push((
            "STONE".into(),
            "Lg/Block;".into(),
            0x0019,
        ));
        let back = Api::load(&api.save());
        assert_eq!(back.classes, api.classes);
    }

    #[test]
    fn descriptors_take_the_mapped_names() {
        let names = HashMap::from([
            ("a".to_owned(), "g/Block".to_owned()),
            ("b$c".to_owned(), "g/Slab$Type".to_owned()),
        ]);
        assert_eq!(
            rename_descriptor("(La;ILb$c;[La;)Lx/Y;", &names),
            "(Lg/Block;ILg/Slab$Type;[Lg/Block;)Lx/Y;"
        );
    }
}
