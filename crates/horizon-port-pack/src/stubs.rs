//! Lane 3's missing API, stood in for. A Java mod names hundreds of Minecraft and loader
//! classes; Horizon's Java API slice holds the ones it implements, and any other class the
//! mod's code names would stop the mod at its first use (`NoClassDefFoundError`, often in
//! the static initialiser that registers every block). [`Refs`] reads what a mod's classes
//! name (classes, the members they call or read, which calls are static, which owners are
//! interfaces, what they extend), and [`stubs`] writes a class file for each class nobody
//! provides: every member the mod names is there and does nothing, answering a default
//! (0, `false`, the object itself for a builder, a fresh stand-in object, an empty string or
//! list). A mod then starts and registers what it has; what the stand-ins swallow is the API
//! Horizon still lacks, and the report lists them.
//!
//! The stand-ins are written from the mod's own references alone, never from Minecraft's
//! classes: they hold names and descriptors the mod's jar already carries, and no code.
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// How a member is reached: `getstatic`/`invokestatic` or through an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reach {
    Static,
    Instance,
}

/// What a mod's classes name, by internal name (`net/minecraft/core/BlockPos`).
#[derive(Debug, Default, Clone)]
pub struct Refs {
    /// Every class named: as a class constant, or in a descriptor.
    pub classes: BTreeSet<String>,
    /// Methods called, by owner: (name, descriptor) and how.
    pub methods: BTreeMap<String, BTreeMap<(String, String), Reach>>,
    /// Fields read or written, by owner: (name, descriptor) and how.
    pub fields: BTreeMap<String, BTreeMap<(String, String), Reach>>,
    /// Owners called as interfaces (`InterfaceMethodref`), or implemented by a class here.
    pub interfaces: BTreeSet<String>,
    /// Classes a class here extends.
    pub extended: BTreeSet<String>,
    /// The types of its lambdas and method references: interfaces, unless the API says else
    /// (a string concatenation's `invokedynamic` gives `String`, never stood in for).
    pub functional: BTreeSet<String>,
    /// Classes the jars declare.
    pub declared: BTreeSet<String>,
    /// What each declared class extends and implements.
    pub parents: BTreeMap<String, (String, Vec<String>)>,
}

/// What Horizon already has of a class the mod names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Known {
    pub interface: bool,
    /// It has a public constructor taking nothing: a stand-in can make one.
    pub plain: bool,
}

/// A constant pool entry, as far as these reads need.
#[derive(Clone, Debug)]
enum Constant {
    Utf8(String),
    Class(u16),
    NameAndType(u16, u16),
    /// Fieldref, Methodref or InterfaceMethodref: tag, class, name and type.
    Member(u8, u16, u16),
    /// reference kind, reference index.
    Handle(u8, u16),
    /// An `invokedynamic` call site's name and type.
    Dynamic(u16),
    Other,
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
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let s = self
            .bytes
            .get(self.at..self.at + n)
            .ok_or("class file cut short")?;
        self.at += n;
        Ok(s)
    }
}

/// The class names in a descriptor (`(Lx/A;[Lx/B;I)Lx/C;`).
pub fn descriptor_classes(descriptor: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = descriptor;
    while let Some(i) = rest.find('L') {
        let after = &rest[i + 1..];
        let Some(end) = after.find(';') else { break };
        out.push(&after[..end]);
        rest = &after[end + 1..];
    }
    out
}

/// A class constant's name as a class: an array's element type, or none for a primitive array.
fn class_of(name: &str) -> Option<&str> {
    let element = name.trim_start_matches('[');
    if element.len() == name.len() {
        return Some(name);
    }
    element.strip_prefix('L')?.strip_suffix(';')
}

/// Bytes an instruction takes, from its opcode at `pc` (`code` from the method's start).
fn instruction_length(code: &[u8], pc: usize) -> Result<usize, String> {
    let op = code[pc];
    let word = |at: usize| -> Result<i32, String> {
        code.get(at..at + 4)
            .map(|b| i32::from_be_bytes([b[0], b[1], b[2], b[3]]))
            .ok_or_else(|| "code cut short".to_owned())
    };
    Ok(match op {
        0x10 | 0x12 | 0x15..=0x19 | 0x36..=0x3a | 0xa9 | 0xbc => 2,
        0x11
        | 0x13
        | 0x14
        | 0x84
        | 0x99..=0xa8
        | 0xb2..=0xb8
        | 0xbb
        | 0xbd
        | 0xc0
        | 0xc1
        | 0xc6
        | 0xc7 => 3,
        0xc5 => 4,
        0xb9 | 0xba | 0xc8 | 0xc9 => 5,
        0xc4 => {
            if code.get(pc + 1) == Some(&0x84) {
                6
            } else {
                4
            }
        }
        0xaa => {
            let base = pc + 1 + (4 - (pc + 1) % 4) % 4;
            let (low, high) = (word(base + 4)?, word(base + 8)?);
            let n =
                usize::try_from(i64::from(high) - i64::from(low) + 1).map_err(|_| "bad switch")?;
            base - pc + 12 + 4 * n
        }
        0xab => {
            let base = pc + 1 + (4 - (pc + 1) % 4) % 4;
            let n = usize::try_from(word(base + 4)?).map_err(|_| "bad switch")?;
            base - pc + 8 + 8 * n
        }
        _ => 1,
    })
}

impl Refs {
    /// Reads one class file of the mod (or of a library it carries).
    pub fn add_class(&mut self, bytes: &[u8]) -> Result<(), String> {
        let mut r = Reader { bytes, at: 0 };
        if r.u4()? != 0xCAFE_BABE {
            return Err("not a class file".into());
        }
        r.take(4)?;
        let count = r.u2()?;
        let mut pool = vec![Constant::Other; usize::from(count)];
        let mut i = 1;
        while i < count {
            let tag = r.u1()?;
            pool[usize::from(i)] = match tag {
                1 => {
                    let n = usize::from(r.u2()?);
                    Constant::Utf8(String::from_utf8_lossy(r.take(n)?).into_owned())
                }
                7 => Constant::Class(r.u2()?),
                9..=11 => Constant::Member(tag, r.u2()?, r.u2()?),
                12 => Constant::NameAndType(r.u2()?, r.u2()?),
                15 => Constant::Handle(r.u1()?, r.u2()?),
                3 | 4 => {
                    r.take(4)?;
                    Constant::Other
                }
                5 | 6 => {
                    r.take(8)?;
                    i += 1;
                    Constant::Other
                }
                8 | 16 | 19 | 20 => {
                    r.take(2)?;
                    Constant::Other
                }
                17 => {
                    r.take(4)?;
                    Constant::Other
                }
                18 => {
                    r.take(2)?;
                    Constant::Dynamic(r.u2()?)
                }
                other => return Err(format!("unknown constant tag {other}")),
            };
            i += 1;
        }
        let utf8 = |k: u16| match pool.get(usize::from(k)) {
            Some(Constant::Utf8(s)) => s.as_str(),
            _ => "",
        };
        let class = |k: u16| match pool.get(usize::from(k)) {
            Some(Constant::Class(n)) => utf8(*n),
            _ => "",
        };
        // Every class constant, and the classes of every descriptor.
        for c in &pool {
            match c {
                Constant::Class(n) => {
                    if let Some(name) = class_of(utf8(*n)) {
                        self.classes.insert(name.to_owned());
                    }
                }
                Constant::NameAndType(_, d) => {
                    for name in descriptor_classes(utf8(*d)) {
                        self.classes.insert(name.to_owned());
                    }
                }
                _ => {}
            }
        }
        // A lambda's or method reference's type is a functional interface.
        for c in &pool {
            if let Constant::Dynamic(nat) = c
                && let Some(Constant::NameAndType(_, d)) = pool.get(usize::from(*nat))
                && let Some((_, ret)) = utf8(*d).split_once(')')
                && let Some(class) = ret.strip_prefix('L').and_then(|r| r.strip_suffix(';'))
            {
                self.functional.insert(class.to_owned());
            }
        }
        // How each member constant is reached.
        let mut reach: HashMap<u16, Reach> = HashMap::new();
        for c in &pool {
            if let Constant::Handle(kind, index) = c {
                reach.insert(
                    *index,
                    if matches!(kind, 2 | 4 | 6) {
                        Reach::Static
                    } else {
                        Reach::Instance
                    },
                );
            }
        }
        r.take(2)?;
        let this = class(r.u2()?).to_owned();
        let parent = r.u2()?;
        self.declared.insert(this.clone());
        if parent != 0 {
            self.extended.insert(class(parent).to_owned());
        }
        let mut implemented = Vec::new();
        for _ in 0..r.u2()? {
            let k = r.u2()?;
            self.interfaces.insert(class(k).to_owned());
            implemented.push(class(k).to_owned());
        }
        self.parents
            .insert(this.clone(), (class(parent).to_owned(), implemented));
        let skip_attributes = |r: &mut Reader| -> Result<(), String> {
            for _ in 0..r.u2()? {
                r.take(2)?;
                let n = r.u4()? as usize;
                r.take(n)?;
            }
            Ok(())
        };
        // The classes of its own fields' and methods' types (an override names its parameters).
        let mut own_types = Vec::new();
        for _ in 0..r.u2()? {
            r.take(4)?;
            own_types.push(r.u2()?);
            skip_attributes(&mut r)?;
        }
        for _ in 0..r.u2()? {
            r.take(4)?;
            own_types.push(r.u2()?);
            for _ in 0..r.u2()? {
                let name = utf8(r.u2()?);
                let n = r.u4()? as usize;
                let body = r.take(n)?;
                if name != "Code" || body.len() < 8 {
                    continue;
                }
                let length = u32::from_be_bytes([body[4], body[5], body[6], body[7]]) as usize;
                let code = body.get(8..8 + length).ok_or("code cut short")?;
                let mut pc = 0;
                while pc < code.len() {
                    let op = code[pc];
                    if matches!(op, 0xb2..=0xb9) && pc + 2 < code.len() {
                        let k = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                        let how = if matches!(op, 0xb2 | 0xb3 | 0xb8) {
                            Reach::Static
                        } else {
                            Reach::Instance
                        };
                        reach.insert(k, how);
                    }
                    pc += instruction_length(code, pc)?;
                }
            }
        }
        for d in own_types {
            for name in descriptor_classes(utf8(d)) {
                self.classes.insert(name.to_owned());
            }
        }
        for (k, how) in reach {
            let Some(Constant::Member(tag, owner, nat)) = pool.get(usize::from(k)) else {
                continue;
            };
            let Some(Constant::NameAndType(name, descriptor)) = pool.get(usize::from(*nat)) else {
                continue;
            };
            let Some(owner) = class_of(class(*owner)) else {
                continue;
            };
            let key = (utf8(*name).to_owned(), utf8(*descriptor).to_owned());
            if *tag == 9 {
                self.fields
                    .entry(owner.to_owned())
                    .or_default()
                    .insert(key, how);
            } else {
                if *tag == 11 {
                    self.interfaces.insert(owner.to_owned());
                }
                self.methods
                    .entry(owner.to_owned())
                    .or_default()
                    .insert(key, how);
            }
        }
        Ok(())
    }

    /// The classes named that the jars do not declare: what Horizon has to provide.
    pub fn wanted(&self) -> impl Iterator<Item = &String> {
        self.classes
            .iter()
            .filter(|c| !self.declared.contains(*c) && !c.is_empty())
    }
}

/// Classes the platform has: never stood in for, even when Horizon's lookup misses one.
pub fn platform(class: &str) -> bool {
    ["java/", "javax/", "jdk/", "sun/", "android/", "dalvik/"]
        .iter()
        .any(|p| class.starts_with(p))
}

const PUBLIC: u16 = 0x0001;
const STATIC: u16 = 0x0008;
const FINAL: u16 = 0x0010;
const SUPER: u16 = 0x0020;
const INTERFACE: u16 = 0x0200;
const ABSTRACT: u16 = 0x0400;
const ENUM: u16 = 0x4000;

#[derive(Default)]
struct Pool {
    bytes: Vec<u8>,
    count: u16,
    seen: HashMap<Vec<u8>, u16>,
}
impl Pool {
    fn add(&mut self, entry: Vec<u8>) -> u16 {
        if let Some(&i) = self.seen.get(&entry) {
            return i;
        }
        self.count += 1;
        let i = self.count;
        self.bytes.extend_from_slice(&entry);
        self.seen.insert(entry, i);
        i
    }
    fn utf8(&mut self, s: &str) -> u16 {
        let mut e = vec![1];
        e.extend_from_slice(&(s.len() as u16).to_be_bytes());
        e.extend_from_slice(s.as_bytes());
        self.add(e)
    }
    fn class(&mut self, name: &str) -> u16 {
        let n = self.utf8(name);
        let mut e = vec![7];
        e.extend_from_slice(&n.to_be_bytes());
        self.add(e)
    }
    fn string(&mut self, s: &str) -> u16 {
        let n = self.utf8(s);
        let mut e = vec![8];
        e.extend_from_slice(&n.to_be_bytes());
        self.add(e)
    }
    fn member(&mut self, tag: u8, owner: &str, name: &str, descriptor: &str) -> u16 {
        let c = self.class(owner);
        let (n, d) = (self.utf8(name), self.utf8(descriptor));
        let mut nat = vec![12];
        nat.extend_from_slice(&n.to_be_bytes());
        nat.extend_from_slice(&d.to_be_bytes());
        let nat = self.add(nat);
        let mut e = vec![tag];
        e.extend_from_slice(&c.to_be_bytes());
        e.extend_from_slice(&nat.to_be_bytes());
        self.add(e)
    }
}

/// A method's argument slots (`this` not counted) and its return type.
fn signature(descriptor: &str) -> (u16, &str) {
    let (args, ret) = descriptor
        .strip_prefix('(')
        .and_then(|d| d.split_once(')'))
        .unwrap_or(("", "V"));
    let mut slots = 0;
    let mut chars = args.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            'J' | 'D' => slots += 2,
            'L' => {
                for c in chars.by_ref() {
                    if c == ';' {
                        break;
                    }
                }
                slots += 1;
            }
            '[' => {
                while chars.peek() == Some(&'[') {
                    chars.next();
                }
                if chars.next() == Some('L') {
                    for c in chars.by_ref() {
                        if c == ';' {
                            break;
                        }
                    }
                }
                slots += 1;
            }
            _ => slots += 1,
        }
    }
    (slots, ret)
}

/// A method a stand-in must not declare: one of `Object`'s final methods (and, for an
/// interface, any public method of `Object`), or one `Enum` makes final.
fn inherited_final(name: &str, descriptor: &str, interface: bool, is_enum: bool) -> bool {
    let object_final = matches!(
        (name, descriptor),
        ("getClass", "()Ljava/lang/Class;")
            | ("notify", "()V")
            | ("notifyAll", "()V")
            | ("wait", "()V" | "(J)V" | "(JI)V")
    );
    let object_public = matches!(
        (name, descriptor),
        ("equals", "(Ljava/lang/Object;)Z")
            | ("hashCode", "()I")
            | ("toString", "()Ljava/lang/String;")
    );
    let enum_final = matches!(
        (name, descriptor),
        ("equals", "(Ljava/lang/Object;)Z")
            | ("hashCode", "()I")
            | ("name", "()Ljava/lang/String;")
            | ("ordinal", "()I")
            | ("compareTo", "(Ljava/lang/Enum;)I")
            | ("getDeclaringClass", "()Ljava/lang/Class;")
            | ("clone", "()Ljava/lang/Object;")
            | ("finalize", "()V")
    );
    object_final || (interface && object_public) || (is_enum && enum_final)
}

/// One stand-in class being written.
struct Stub<'a> {
    name: &'a str,
    pool: Pool,
    methods: Vec<u8>,
    method_count: u16,
}

/// Empty values the platform gives for its common return types: the type, then the static
/// method making one (its owner, name and descriptor, and whether the owner is an interface).
const EMPTY: [(&str, &str, &str, &str, bool); 8] = [
    (
        "Ljava/util/List;",
        "java/util/Collections",
        "emptyList",
        "()Ljava/util/List;",
        false,
    ),
    (
        "Ljava/util/Collection;",
        "java/util/Collections",
        "emptyList",
        "()Ljava/util/List;",
        false,
    ),
    (
        "Ljava/lang/Iterable;",
        "java/util/Collections",
        "emptyList",
        "()Ljava/util/List;",
        false,
    ),
    (
        "Ljava/util/Set;",
        "java/util/Collections",
        "emptySet",
        "()Ljava/util/Set;",
        false,
    ),
    (
        "Ljava/util/Map;",
        "java/util/Collections",
        "emptyMap",
        "()Ljava/util/Map;",
        false,
    ),
    (
        "Ljava/util/Optional;",
        "java/util/Optional",
        "empty",
        "()Ljava/util/Optional;",
        false,
    ),
    (
        "Ljava/util/stream/Stream;",
        "java/util/stream/Stream",
        "empty",
        "()Ljava/util/stream/Stream;",
        true,
    ),
    (
        "Ljava/util/Iterator;",
        "java/util/Collections",
        "emptyIterator",
        "()Ljava/util/Iterator;",
        false,
    ),
];

impl Stub<'_> {
    fn method(&mut self, access: u16, name: &str, descriptor: &str, code: &[u8], locals: u16) {
        let (n, d) = (self.pool.utf8(name), self.pool.utf8(descriptor));
        let attribute = self.pool.utf8("Code");
        self.methods.extend_from_slice(&access.to_be_bytes());
        self.methods.extend_from_slice(&n.to_be_bytes());
        self.methods.extend_from_slice(&d.to_be_bytes());
        self.methods.extend_from_slice(&1u16.to_be_bytes());
        self.methods.extend_from_slice(&attribute.to_be_bytes());
        self.methods
            .extend_from_slice(&(12 + code.len() as u32).to_be_bytes());
        // Room for the arguments a forwarding constructor pushes, and the few a body needs.
        let stack = locals.saturating_add(2).max(8);
        self.methods.extend_from_slice(&stack.to_be_bytes());
        self.methods.extend_from_slice(&locals.to_be_bytes());
        self.methods
            .extend_from_slice(&(code.len() as u32).to_be_bytes());
        self.methods.extend_from_slice(code);
        self.methods.extend_from_slice(&[0, 0, 0, 0]);
        self.method_count += 1;
    }
    /// Code that leaves a fresh object of class `class` on the stack (`new`, `dup`,
    /// `invokespecial <init>()V`).
    fn fresh(&mut self, class: &str) -> Vec<u8> {
        let c = self.pool.class(class);
        let init = self.pool.member(10, class, "<init>", "()V");
        let mut code = vec![0xbb];
        code.extend_from_slice(&c.to_be_bytes());
        code.push(0x59);
        code.push(0xb7);
        code.extend_from_slice(&init.to_be_bytes());
        code
    }
    /// A body answering the default for return type `ret`.
    fn answer(&mut self, ret: &str, instance: bool, makeable: &dyn Fn(&str) -> bool) -> Vec<u8> {
        match ret {
            "V" => vec![0xb1],
            "Z" | "B" | "C" | "S" | "I" => vec![0x03, 0xac],
            "J" => vec![0x09, 0xad],
            "F" => vec![0x0b, 0xae],
            "D" => vec![0x0e, 0xaf],
            "Ljava/lang/String;" => {
                let s = self.pool.string("");
                let mut code = vec![0x13];
                code.extend_from_slice(&s.to_be_bytes());
                code.push(0xb0);
                code
            }
            _ if ret.starts_with('[') => {
                // A zero-length array of the element type.
                let element = &ret[1..];
                let mut code = vec![0x03];
                let primitive = match element {
                    "Z" => Some(4),
                    "C" => Some(5),
                    "F" => Some(6),
                    "D" => Some(7),
                    "B" => Some(8),
                    "S" => Some(9),
                    "I" => Some(10),
                    "J" => Some(11),
                    _ => None,
                };
                match primitive {
                    Some(t) => code.extend_from_slice(&[0xbc, t]),
                    None => {
                        let class = element
                            .strip_prefix('L')
                            .and_then(|e| e.strip_suffix(';'))
                            .unwrap_or(element);
                        let c = self.pool.class(class);
                        code.push(0xbd);
                        code.extend_from_slice(&c.to_be_bytes());
                    }
                }
                code.push(0xb0);
                code
            }
            _ => {
                let class = ret
                    .strip_prefix('L')
                    .and_then(|r| r.strip_suffix(';'))
                    .unwrap_or("");
                if instance && class == self.name {
                    // A builder answers itself.
                    return vec![0x2a, 0xb0];
                }
                if let Some((_, owner, name, descriptor, iface)) = EMPTY.iter().find(|e| e.0 == ret)
                {
                    let m = self
                        .pool
                        .member(if *iface { 11 } else { 10 }, owner, name, descriptor);
                    let mut code = vec![0xb8];
                    code.extend_from_slice(&m.to_be_bytes());
                    code.push(0xb0);
                    return code;
                }
                if !class.is_empty() && makeable(class) {
                    let mut code = self.fresh(class);
                    code.push(0xb0);
                    return code;
                }
                vec![0x01, 0xb0]
            }
        }
    }
}

/// What Horizon has, for [`stubs`].
pub struct Provided<'a> {
    /// A class Horizon's API slice holds.
    pub class: &'a dyn Fn(&str) -> Option<Known>,
    /// Whether a class Horizon holds has a constructor with this descriptor.
    pub constructor: &'a dyn Fn(&str, &str) -> bool,
    /// Whether a class Horizon holds declares (or inherits) this method or field.
    pub member: &'a dyn Fn(&str, &str, &str) -> bool,
    /// Every superclass and interface of a class Horizon holds, as Horizon declares them.
    pub ancestors: &'a dyn Fn(&str) -> BTreeSet<String>,
}

/// The stand-ins, and what they could not stand in for.
#[derive(Debug, Default)]
pub struct Stubs {
    /// Class files by internal name.
    pub classes: BTreeMap<String, Vec<u8>>,
    /// What each of Horizon's classes the mod names lacks: for [`patch`].
    pub patches: BTreeMap<String, Patch>,
    /// The classes a stand-in body can make with `new` (for [`patch`]).
    pub makeable: BTreeSet<String>,
    /// Members a mod names on a class Horizon holds that it lacks (`owner.name descriptor`),
    /// and the game's supertypes Horizon's class lacks (`owner is a supertype`): the API
    /// slice's own gaps, which no stand-in can fill.
    pub unmet: BTreeSet<String>,
}

/// Members and interfaces to add to one of Horizon's own classes, so that the mod's code
/// finds every member it names there (answering defaults until Horizon writes them).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Patch {
    pub interfaces: BTreeSet<String>,
    /// (name, descriptor) and how.
    pub methods: BTreeMap<(String, String), Reach>,
    pub fields: BTreeMap<(String, String), Reach>,
}

/// One member a stand-in declares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Member {
    reach: Reach,
}

/// Code loading a method's arguments (after `this` in slot 0 when `instance`).
fn load_arguments(descriptor: &str, instance: bool) -> Vec<u8> {
    let args = descriptor
        .strip_prefix('(')
        .and_then(|d| d.split_once(')'))
        .map_or("", |(a, _)| a);
    let mut code = Vec::new();
    let mut slot: u16 = u16::from(instance);
    let mut chars = args.chars().peekable();
    let load = |code: &mut Vec<u8>, op: u8, slot: u16| {
        if slot <= 255 {
            code.extend_from_slice(&[op, slot as u8]);
        } else {
            code.extend_from_slice(&[0xc4, op]);
            code.extend_from_slice(&slot.to_be_bytes());
        }
    };
    while let Some(c) = chars.next() {
        match c {
            'Z' | 'B' | 'C' | 'S' | 'I' => {
                load(&mut code, 0x15, slot);
                slot += 1;
            }
            'J' => {
                load(&mut code, 0x16, slot);
                slot += 2;
            }
            'F' => {
                load(&mut code, 0x17, slot);
                slot += 1;
            }
            'D' => {
                load(&mut code, 0x18, slot);
                slot += 2;
            }
            'L' => {
                for c in chars.by_ref() {
                    if c == ';' {
                        break;
                    }
                }
                load(&mut code, 0x19, slot);
                slot += 1;
            }
            '[' => {
                while chars.peek() == Some(&'[') {
                    chars.next();
                }
                if chars.next() == Some('L') {
                    for c in chars.by_ref() {
                        if c == ';' {
                            break;
                        }
                    }
                }
                load(&mut code, 0x19, slot);
                slot += 1;
            }
            _ => {}
        }
    }
    code
}

/// Class files standing in for every class `refs` names that neither the jars nor Horizon
/// provide, with the hierarchy `api` gives them (an empty API: each extends `Object`).
pub fn stubs(refs: &Refs, provided: &Provided, api: &crate::api::Api) -> Stubs {
    let known = provided.class;
    let absent =
        |c: &str| !c.is_empty() && !platform(c) && !refs.declared.contains(c) && known(c).is_none();
    // What to write: every class named and nobody's, and the missing ancestors of those.
    let mut write: BTreeSet<String> = refs.wanted().filter(|c| absent(c)).cloned().collect();
    let mut queue: Vec<String> = write.iter().cloned().collect();
    // The mod's own classes' ancestors count too: `MyBlock extends SlabBlock`.
    queue.extend(refs.declared.iter().cloned());
    while let Some(c) = queue.pop() {
        let Some(info) = api.classes.get(&c) else {
            continue;
        };
        for up in std::iter::once(&info.parent).chain(&info.interfaces) {
            if absent(up) && write.insert(up.clone()) {
                queue.push(up.clone());
            }
        }
    }
    let interface = |c: &str| match api.classes.get(c) {
        Some(info) => info.is_interface(),
        None => {
            (refs.interfaces.contains(c) || refs.functional.contains(c))
                && !refs.extended.contains(c)
        }
    };
    // A class a stand-in can make with `new`: a stand-in class, or one of Horizon's with a
    // public constructor taking nothing.
    let makeable = |c: &str| {
        if write.contains(c) {
            !interface(c)
        } else {
            known(c).is_some_and(|k| k.plain && !k.interface)
        }
    };
    // Each member named, on the class declaring it (as the API resolves it), or on the class
    // it was named on when the API doesn't know it.
    let mut methods: BTreeMap<String, BTreeMap<(String, String), Member>> = BTreeMap::new();
    let mut fields: BTreeMap<String, BTreeMap<(String, String), Member>> = BTreeMap::new();
    let mut unmet = BTreeSet::new();
    let mut patches: BTreeMap<String, Patch> = BTreeMap::new();
    let mut place = |table: &mut BTreeMap<String, BTreeMap<(String, String), Member>>,
                     owner: &str,
                     name: &str,
                     descriptor: &str,
                     reach: Reach,
                     declared: Option<(&str, u16)>,
                     field: bool| {
        let (at, reach) = match declared {
            Some((d, access)) => (
                d,
                if access & STATIC != 0 {
                    Reach::Static
                } else {
                    Reach::Instance
                },
            ),
            // Unknown to the API: a member named on one of the mod's own classes goes to the
            // first ancestor that isn't.
            None if refs.declared.contains(owner) => {
                let mut at = owner;
                let mut steps = 0;
                while let Some((parent, _)) = refs.parents.get(at) {
                    if parent.is_empty() || steps > 64 {
                        break;
                    }
                    at = parent.as_str();
                    steps += 1;
                }
                (at, reach)
            }
            None => (owner, reach),
        };
        if write.contains(at) {
            table
                .entry(at.to_owned())
                .or_default()
                .insert((name.to_owned(), descriptor.to_owned()), Member { reach });
        } else if known(at).is_some() && !(provided.member)(at, name, descriptor) {
            let kind = if field { "field " } else { "" };
            unmet.insert(format!("{kind}{at}.{name} {descriptor}"));
            let patch = patches.entry(at.to_owned()).or_default();
            let key = (name.to_owned(), descriptor.to_owned());
            if field {
                patch.fields.insert(key, reach);
            } else {
                patch.methods.insert(key, reach);
            }
        }
    };
    // A member named on one of the mod's classes resolves through its ancestors: the first
    // that isn't the mod's own is where the API's resolution starts.
    let outside = |owner: &str| -> String {
        let mut at = owner.to_owned();
        let mut steps = 0;
        while let Some((parent, _)) = refs.parents.get(&at) {
            if parent.is_empty() || steps > 64 {
                break;
            }
            at = parent.clone();
            steps += 1;
        }
        at
    };
    for (owner, members) in &refs.methods {
        for ((name, descriptor), reach) in members {
            if platform(owner) || name == "<clinit>" {
                continue;
            }
            let declared = if name == "<init>" {
                api.classes
                    .get(owner)
                    .and_then(|c| c.methods.get(&(name.clone(), descriptor.clone())))
                    .map(|a| (owner.as_str(), *a))
            } else if refs.declared.contains(owner) {
                let start = outside(owner);
                api.declaring_method(&start, name, descriptor)
            } else {
                api.declaring_method(owner, name, descriptor)
            };
            place(
                &mut methods,
                owner,
                name,
                descriptor,
                *reach,
                declared,
                false,
            );
        }
    }
    for (owner, members) in &refs.fields {
        for ((name, descriptor), reach) in members {
            if platform(owner) {
                continue;
            }
            let declared = if refs.declared.contains(owner) {
                api.declaring_field(&outside(owner), name, descriptor)
            } else {
                api.declaring_field(owner, name, descriptor)
            };
            place(&mut fields, owner, name, descriptor, *reach, declared, true);
        }
    }
    // A stand-in's constructor reaches its parent's with the same arguments, so that
    // `new SlabBlock(properties)` still gives `Block` its properties: the parent stand-in gets
    // that constructor too, up to a class Horizon holds.
    let mut constructors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (owner, members) in &methods {
        for (name, descriptor) in members.keys() {
            if name == "<init>" {
                constructors
                    .entry(owner.clone())
                    .or_default()
                    .insert(descriptor.clone());
            }
        }
    }
    let parent_of = |c: &str| -> String {
        match api.classes.get(c) {
            Some(info) if !info.parent.is_empty() && !info.is_interface() => info.parent.clone(),
            _ => "java/lang/Object".to_owned(),
        }
    };
    let mut pending: Vec<(String, String)> = constructors
        .iter()
        .flat_map(|(c, ds)| ds.iter().map(move |d| (c.clone(), d.clone())))
        .collect();
    while let Some((c, d)) = pending.pop() {
        let p = parent_of(&c);
        if write.contains(&p)
            && api
                .classes
                .get(&p)
                .is_some_and(|i| i.methods.contains_key(&("<init>".to_owned(), d.clone())))
            && constructors.entry(p.clone()).or_default().insert(d.clone())
        {
            pending.push((p, d));
        }
    }
    let none = BTreeMap::new();
    let mut out = BTreeMap::new();
    for name in &write {
        let name = name.as_str();
        let info = api.classes.get(name);
        let own_methods = methods.get(name).unwrap_or(&none);
        let own_fields = fields.get(name).unwrap_or(&none);
        let is_interface = interface(name);
        let own = format!("L{name};");
        // Its enum constants: all of them when the API knows the enum, else the static fields
        // of its own type named on a class asked for `ordinal()` or `values()`.
        let constants: Vec<String> = match info {
            Some(i) if i.is_enum() => i.constants().map(str::to_owned).collect(),
            Some(_) => Vec::new(),
            None => {
                let guessed: Vec<String> = own_fields
                    .iter()
                    .filter(|((_, d), m)| m.reach == Reach::Static && *d == own)
                    .map(|((n, _), _)| n.clone())
                    .collect();
                let asked = own_methods
                    .keys()
                    .any(|(n, d)| (n == "ordinal" && d == "()I") || n == "values");
                if asked { guessed } else { Vec::new() }
            }
        };
        let is_enum = !is_interface && !constants.is_empty();
        let parent = if is_enum {
            "java/lang/Enum".to_owned()
        } else if is_interface {
            "java/lang/Object".to_owned()
        } else {
            parent_of(name)
        };
        let mut stub = Stub {
            name,
            pool: Pool::default(),
            methods: Vec::new(),
            method_count: 0,
        };
        let this = stub.pool.class(name);
        let parent_index = stub.pool.class(&parent);
        let interfaces: Vec<u16> = info
            .map(|i| i.interfaces.clone())
            .unwrap_or_default()
            .iter()
            .filter(|i| platform(i) || write.contains(*i) || known(i).is_some_and(|k| k.interface))
            .map(|i| stub.pool.class(i))
            .collect();
        // Fields: those named, and every enum constant.
        let mut field_list: BTreeMap<(String, String), Member> = own_fields.clone();
        for c in &constants {
            field_list.insert(
                (c.clone(), own.clone()),
                Member {
                    reach: Reach::Static,
                },
            );
        }
        let mut field_bytes = Vec::new();
        let mut field_count = 0u16;
        for ((field, descriptor), m) in &field_list {
            let mut access = PUBLIC;
            if m.reach == Reach::Static || is_interface {
                access |= STATIC;
            }
            if is_interface {
                access |= FINAL;
            }
            if is_enum && constants.contains(field) && *descriptor == own {
                access |= FINAL | ENUM;
            }
            let (n, d) = (stub.pool.utf8(field), stub.pool.utf8(descriptor));
            field_bytes.extend_from_slice(&access.to_be_bytes());
            field_bytes.extend_from_slice(&n.to_be_bytes());
            field_bytes.extend_from_slice(&d.to_be_bytes());
            field_bytes.extend_from_slice(&0u16.to_be_bytes());
            field_count += 1;
        }
        // Constructors.
        if !is_interface {
            if is_enum {
                let init =
                    stub.pool
                        .member(10, "java/lang/Enum", "<init>", "(Ljava/lang/String;I)V");
                let mut code = vec![0x2a, 0x2b, 0x1c, 0xb7];
                code.extend_from_slice(&init.to_be_bytes());
                code.push(0xb1);
                stub.method(0x0002, "<init>", "(Ljava/lang/String;I)V", &code, 3);
            } else {
                let mut wanted = constructors.get(name).cloned().unwrap_or_default();
                wanted.insert("()V".to_owned());
                for descriptor in wanted {
                    let (slots, _) = signature(&descriptor);
                    let forward = descriptor != "()V"
                        && if write.contains(&parent) {
                            constructors
                                .get(&parent)
                                .is_some_and(|ds| ds.contains(&descriptor))
                        } else {
                            (provided.constructor)(&parent, &descriptor)
                        };
                    let mut code = vec![0x2a];
                    let init = if forward {
                        code.extend_from_slice(&load_arguments(&descriptor, true));
                        stub.pool.member(10, &parent, "<init>", &descriptor)
                    } else {
                        stub.pool.member(10, &parent, "<init>", "()V")
                    };
                    code.push(0xb7);
                    code.extend_from_slice(&init.to_be_bytes());
                    code.push(0xb1);
                    stub.method(PUBLIC, "<init>", &descriptor, &code, slots + 1);
                }
            }
        }
        // Methods.
        let mut has_values = false;
        for ((method, descriptor), m) in own_methods {
            if method.starts_with('<') || inherited_final(method, descriptor, is_interface, is_enum)
            {
                continue;
            }
            if is_enum && method == "values" {
                has_values = true;
                continue;
            }
            let (slots, ret) = signature(descriptor);
            let instance = m.reach == Reach::Instance;
            let code = stub.answer(ret, instance, &makeable);
            let access = if instance { PUBLIC } else { PUBLIC | STATIC };
            stub.method(
                access,
                method,
                descriptor,
                &code,
                slots + u16::from(instance),
            );
        }
        if is_enum {
            // values(): the constants in order (`switch` tables index it by ordinal()).
            let mut code = Vec::new();
            let n = constants.len().min(32767) as u16;
            code.push(0x11);
            code.extend_from_slice(&n.to_be_bytes());
            code.push(0xbd);
            code.extend_from_slice(&this.to_be_bytes());
            for (i, constant) in constants.iter().enumerate().take(usize::from(n)) {
                let f = stub.pool.member(9, name, constant, &own);
                code.extend_from_slice(&[0x59, 0x11]);
                code.extend_from_slice(&(i as u16).to_be_bytes());
                code.push(0xb2);
                code.extend_from_slice(&f.to_be_bytes());
                code.push(0x53);
            }
            code.push(0xb0);
            let _ = has_values;
            stub.method(PUBLIC | STATIC, "values", &format!("()[{own}"), &code, 0);
        }
        // Static fields that hold objects: the enum's constants in order, else a fresh
        // stand-in or an empty collection.
        let mut clinit = Vec::new();
        for ((field, descriptor), m) in &field_list {
            if m.reach != Reach::Static && !is_interface {
                continue;
            }
            let Some(class) = descriptor
                .strip_prefix('L')
                .and_then(|d| d.strip_suffix(';'))
            else {
                continue;
            };
            let f = stub.pool.member(9, name, field, descriptor);
            if is_enum && class == name && constants.contains(field) {
                let i = constants.iter().position(|c| c == field).unwrap_or(0) as u16;
                let s = stub.pool.string(field);
                let init = stub
                    .pool
                    .member(10, name, "<init>", "(Ljava/lang/String;I)V");
                clinit.push(0xbb);
                clinit.extend_from_slice(&this.to_be_bytes());
                clinit.extend_from_slice(&[0x59, 0x13]);
                clinit.extend_from_slice(&s.to_be_bytes());
                clinit.push(0x11);
                clinit.extend_from_slice(&i.to_be_bytes());
                clinit.push(0xb7);
                clinit.extend_from_slice(&init.to_be_bytes());
            } else if makeable(class) && !(is_enum && class == name) {
                let code = stub.fresh(class);
                clinit.extend_from_slice(&code);
            } else if let Some((_, owner, method, d, iface)) =
                EMPTY.iter().find(|e| e.0 == descriptor)
            {
                let m = stub
                    .pool
                    .member(if *iface { 11 } else { 10 }, owner, method, d);
                clinit.push(0xb8);
                clinit.extend_from_slice(&m.to_be_bytes());
            } else {
                continue;
            }
            clinit.push(0xb3);
            clinit.extend_from_slice(&f.to_be_bytes());
        }
        if !clinit.is_empty() {
            clinit.push(0xb1);
            stub.method(STATIC, "<clinit>", "()V", &clinit, 0);
        }
        let mut access = PUBLIC;
        if is_interface {
            access |= INTERFACE | ABSTRACT;
        } else {
            access |= SUPER;
        }
        if is_enum {
            access |= ENUM;
        }
        let mut class = vec![0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 52];
        class.extend_from_slice(&(stub.pool.count + 1).to_be_bytes());
        class.extend_from_slice(&stub.pool.bytes);
        class.extend_from_slice(&access.to_be_bytes());
        class.extend_from_slice(&this.to_be_bytes());
        class.extend_from_slice(&parent_index.to_be_bytes());
        class.extend_from_slice(&(interfaces.len() as u16).to_be_bytes());
        for i in &interfaces {
            class.extend_from_slice(&i.to_be_bytes());
        }
        class.extend_from_slice(&field_count.to_be_bytes());
        class.extend_from_slice(&field_bytes);
        class.extend_from_slice(&stub.method_count.to_be_bytes());
        class.extend_from_slice(&stub.methods);
        class.extend_from_slice(&0u16.to_be_bytes());
        out.insert(name.to_owned(), class);
    }
    // A class Horizon holds that misses one of the game's supertypes: the mod's code passing
    // one where the supertype is asked for would not verify.
    for c in refs.classes.iter().filter(|c| known(c).is_some()) {
        let Some(info) = api.classes.get(c) else {
            continue;
        };
        let held = (provided.ancestors)(c);
        let mut queue: Vec<&str> = std::iter::once(info.parent.as_str())
            .chain(info.interfaces.iter().map(String::as_str))
            .collect();
        let mut seen = BTreeSet::new();
        while let Some(up) = queue.pop() {
            if up.is_empty() || platform(up) || !seen.insert(up) {
                continue;
            }
            if !held.contains(up) {
                unmet.insert(format!("{c} is a {up}"));
                let interface = api.classes.get(up).is_some_and(|i| i.is_interface());
                if interface && (write.contains(up) || known(up).is_some_and(|k| k.interface)) {
                    patches
                        .entry(c.clone())
                        .or_default()
                        .interfaces
                        .insert(up.to_owned());
                }
            }
            if let Some(i) = api.classes.get(up) {
                queue.push(i.parent.as_str());
                queue.extend(i.interfaces.iter().map(String::as_str));
            }
        }
    }
    let makeable = write
        .iter()
        .filter(|c| !interface(c))
        .cloned()
        .chain(
            refs.classes
                .iter()
                .filter(|c| known(c).is_some_and(|k| k.plain && !k.interface))
                .cloned(),
        )
        .collect();
    Stubs {
        classes: out,
        patches,
        makeable,
        unmet,
    }
}

/// One of Horizon's class files with `patch`'s interfaces and members added: each member
/// answers the default a stand-in would, and a static field holding an object gets a fresh
/// one (the class's own static initialiser, renamed, runs after).
pub fn patch(bytes: &[u8], patch: &Patch, makeable: &BTreeSet<String>) -> Result<Vec<u8>, String> {
    let mut r = Reader { bytes, at: 0 };
    if r.u4()? != 0xCAFE_BABE {
        return Err("not a class file".into());
    }
    r.take(4)?;
    let count = r.u2()?;
    let pool_start = r.at;
    let mut utf8: HashMap<u16, String> = HashMap::new();
    let mut classes: HashMap<u16, u16> = HashMap::new();
    let mut i = 1;
    while i < count {
        let tag = r.u1()?;
        match tag {
            1 => {
                let n = usize::from(r.u2()?);
                let s = String::from_utf8_lossy(r.take(n)?).into_owned();
                utf8.insert(i, s);
            }
            7 => {
                let k = r.u2()?;
                classes.insert(i, k);
            }
            8 | 16 | 19 | 20 => {
                r.take(2)?;
            }
            15 => {
                r.take(3)?;
            }
            3 | 4 | 9..=12 | 17 | 18 => {
                r.take(4)?;
            }
            5 | 6 => {
                r.take(8)?;
                i += 1;
            }
            other => return Err(format!("unknown constant tag {other}")),
        }
        i += 1;
    }
    let pool_end = r.at;
    let access = r.u2()?;
    let this = r.u2()?;
    let parent = r.u2()?;
    let name = classes
        .get(&this)
        .and_then(|n| utf8.get(n))
        .cloned()
        .unwrap_or_default();
    let interface_count = r.u2()?;
    let interfaces_at = r.at;
    let mut own_interfaces = BTreeSet::new();
    for _ in 0..interface_count {
        let k = r.u2()?;
        if let Some(n) = classes.get(&k).and_then(|n| utf8.get(n)) {
            own_interfaces.insert(n.clone());
        }
    }
    let interfaces_end = r.at;
    let skip_member = |r: &mut Reader| -> Result<(u16, u16, u16), String> {
        let (a, n, d) = (r.u2()?, r.u2()?, r.u2()?);
        for _ in 0..r.u2()? {
            r.take(2)?;
            let len = r.u4()? as usize;
            r.take(len)?;
        }
        Ok((a, n, d))
    };
    let field_count = r.u2()?;
    let fields_at = r.at;
    let mut own_fields = BTreeSet::new();
    for _ in 0..field_count {
        let (_, n, d) = skip_member(&mut r)?;
        own_fields.insert((
            utf8.get(&n).cloned().unwrap_or_default(),
            utf8.get(&d).cloned().unwrap_or_default(),
        ));
    }
    let fields_end = r.at;
    let method_count = r.u2()?;
    let methods_at = r.at;
    let mut own_methods = BTreeSet::new();
    // The static initialiser's name index, to rename it.
    let mut clinit_name_at = None;
    for _ in 0..method_count {
        let start = r.at;
        let (_, n, d) = skip_member(&mut r)?;
        let (method, descriptor) = (
            utf8.get(&n).cloned().unwrap_or_default(),
            utf8.get(&d).cloned().unwrap_or_default(),
        );
        if method == "<clinit>" {
            clinit_name_at = Some(start + 2);
        }
        own_methods.insert((method, descriptor));
    }
    let methods_end = r.at;
    let is_interface = access & INTERFACE != 0;
    let is_enum = access & ENUM != 0;
    let mut stub = Stub {
        name: &name,
        pool: Pool {
            count: count - 1,
            ..Pool::default()
        },
        methods: Vec::new(),
        method_count: 0,
    };
    let is_makeable = |c: &str| makeable.contains(c);
    // Interfaces.
    let mut new_interfaces = Vec::new();
    for i in &patch.interfaces {
        if !own_interfaces.contains(i) {
            new_interfaces.push(stub.pool.class(i));
        }
    }
    // Fields.
    let mut field_bytes = Vec::new();
    let mut new_fields = 0u16;
    let mut clinit = Vec::new();
    for ((field, descriptor), reach) in &patch.fields {
        if own_fields.contains(&(field.clone(), descriptor.clone())) {
            continue;
        }
        let mut a = PUBLIC;
        if *reach == Reach::Static || is_interface {
            a |= STATIC;
        }
        if is_interface {
            a |= FINAL;
        }
        let (n, d) = (stub.pool.utf8(field), stub.pool.utf8(descriptor));
        field_bytes.extend_from_slice(&a.to_be_bytes());
        field_bytes.extend_from_slice(&n.to_be_bytes());
        field_bytes.extend_from_slice(&d.to_be_bytes());
        field_bytes.extend_from_slice(&0u16.to_be_bytes());
        new_fields += 1;
        if a & STATIC != 0
            && let Some(class) = descriptor
                .strip_prefix('L')
                .and_then(|d| d.strip_suffix(';'))
        {
            let f = stub.pool.member(9, &name, field, descriptor);
            if is_makeable(class) {
                let code = stub.fresh(class);
                clinit.extend_from_slice(&code);
            } else if let Some((_, owner, method, d, iface)) =
                EMPTY.iter().find(|e| e.0 == descriptor)
            {
                let m = stub
                    .pool
                    .member(if *iface { 11 } else { 10 }, owner, method, d);
                clinit.push(0xb8);
                clinit.extend_from_slice(&m.to_be_bytes());
            } else {
                continue;
            }
            clinit.push(0xb3);
            clinit.extend_from_slice(&f.to_be_bytes());
        }
    }
    // Methods. A constructor the class lacks starts the object as its parent's taking nothing
    // does (Horizon's own classes keep one).
    let parent_name = classes
        .get(&parent)
        .and_then(|n| utf8.get(n))
        .cloned()
        .unwrap_or_else(|| "java/lang/Object".to_owned());
    for ((method, descriptor), reach) in &patch.methods {
        if own_methods.contains(&(method.clone(), descriptor.clone()))
            || method == "<clinit>"
            || inherited_final(method, descriptor, is_interface, is_enum)
        {
            continue;
        }
        if method == "<init>" {
            if is_interface || is_enum {
                continue;
            }
            let (slots, _) = signature(descriptor);
            let init = stub.pool.member(10, &parent_name, "<init>", "()V");
            let mut code = vec![0x2a, 0xb7];
            code.extend_from_slice(&init.to_be_bytes());
            code.push(0xb1);
            stub.method(PUBLIC, "<init>", descriptor, &code, slots + 1);
            continue;
        }
        let (slots, ret) = signature(descriptor);
        let instance = *reach == Reach::Instance;
        let code = stub.answer(ret, instance, &is_makeable);
        let a = if instance { PUBLIC } else { PUBLIC | STATIC };
        stub.method(a, method, descriptor, &code, slots + u16::from(instance));
    }
    // The class's own static initialiser runs after the new fields are filled.
    let mut renamed = None;
    if !clinit.is_empty() {
        if clinit_name_at.is_some() {
            let renamed_name = stub.pool.utf8("horizon$clinit");
            renamed = Some(renamed_name);
            let own = stub.pool.member(
                if is_interface { 11 } else { 10 },
                &name,
                "horizon$clinit",
                "()V",
            );
            clinit.push(0xb8);
            clinit.extend_from_slice(&own.to_be_bytes());
        }
        clinit.push(0xb1);
        stub.method(STATIC, "<clinit>", "()V", &clinit, 0);
    }
    let mut out = bytes[..8].to_vec();
    out.extend_from_slice(&(stub.pool.count + 1).to_be_bytes());
    out.extend_from_slice(&bytes[pool_start..pool_end]);
    out.extend_from_slice(&stub.pool.bytes);
    out.extend_from_slice(&access.to_be_bytes());
    out.extend_from_slice(&this.to_be_bytes());
    out.extend_from_slice(&parent.to_be_bytes());
    out.extend_from_slice(&(interface_count + new_interfaces.len() as u16).to_be_bytes());
    out.extend_from_slice(&bytes[interfaces_at..interfaces_end]);
    for i in &new_interfaces {
        out.extend_from_slice(&i.to_be_bytes());
    }
    out.extend_from_slice(&(field_count + new_fields).to_be_bytes());
    out.extend_from_slice(&bytes[fields_at..fields_end]);
    out.extend_from_slice(&field_bytes);
    out.extend_from_slice(&(method_count + stub.method_count).to_be_bytes());
    let mut methods = bytes[methods_at..methods_end].to_vec();
    if let (Some(at), Some(index)) = (clinit_name_at, renamed) {
        // The old initialiser, now `private static horizon$clinit()V`.
        let at = at - methods_at;
        methods[at..at + 2].copy_from_slice(&index.to_be_bytes());
        let flags = if is_interface { 0x0009 } else { 0x000a };
        methods[at - 2..at].copy_from_slice(&(flags as u16).to_be_bytes());
    }
    out.extend_from_slice(&methods);
    out.extend_from_slice(&stub.methods);
    out.extend_from_slice(&bytes[methods_end..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Api, ApiClass};

    /// A mod's view of the API, as `add_class` would have read it from its code.
    fn refs() -> Refs {
        let mut refs = Refs::default();
        let mut method = |owner: &str, name: &str, d: &str, how| {
            refs.classes.insert(owner.to_owned());
            refs.methods
                .entry(owner.to_owned())
                .or_default()
                .insert((name.to_owned(), d.to_owned()), how);
        };
        method("x/Props", "of", "()Lx/Props;", Reach::Static);
        method("x/Props", "strength", "(FF)Lx/Props;", Reach::Instance);
        method("x/Props", "light", "()I", Reach::Instance);
        method("x/Level", "server", "()Lx/Server;", Reach::Instance);
        method("x/Level", "players", "()Ljava/util/List;", Reach::Instance);
        method("x/Dir", "ordinal", "()I", Reach::Instance);
        method("x/Dir", "values", "()[Lx/Dir;", Reach::Static);
        method(
            "x/Handler",
            "handle",
            "(Ljava/lang/Object;)V",
            Reach::Instance,
        );
        refs.interfaces.insert("x/Handler".into());
        for (owner, field, d) in [
            ("x/Dir", "NORTH", "Lx/Dir;"),
            ("x/Dir", "EAST", "Lx/Dir;"),
            ("x/Colors", "STONE", "Lx/Server;"),
        ] {
            refs.classes.insert(owner.to_owned());
            refs.fields
                .entry(owner.to_owned())
                .or_default()
                .insert((field.to_owned(), d.to_owned()), Reach::Static);
        }
        refs.classes.insert("x/Server".into());
        refs.classes.insert("java/lang/String".into());
        refs.classes.insert("x/Known".into());
        refs.classes.insert("my/Mod".into());
        refs.declared.insert("my/Mod".into());
        refs
    }

    fn run(refs: &Refs, api: &Api) -> Stubs {
        let class = |c: &str| {
            (c == "x/Known").then_some(Known {
                interface: false,
                plain: true,
            })
        };
        let constructor = |_: &str, _: &str| false;
        let member = |_: &str, _: &str, _: &str| false;
        let ancestors = |_: &str| BTreeSet::new();
        stubs(
            refs,
            &Provided {
                class: &class,
                constructor: &constructor,
                member: &member,
                ancestors: &ancestors,
            },
            api,
        )
    }

    #[test]
    fn stands_in_only_for_what_nobody_provides() {
        let out = run(&refs(), &Api::default());
        let names: Vec<&str> = out.classes.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "x/Colors",
                "x/Dir",
                "x/Handler",
                "x/Level",
                "x/Props",
                "x/Server"
            ]
        );
    }

    #[test]
    fn a_stand_in_reads_back_with_the_members_it_was_asked_for() {
        let out = run(&refs(), &Api::default());
        let mut back = Refs::default();
        back.add_class(&out.classes["x/Props"]).unwrap();
        assert!(back.declared.contains("x/Props"));
        // The factory makes a fresh object, so it calls the constructor.
        let own = &back.methods["x/Props"];
        assert_eq!(
            own.get(&("<init>".to_owned(), "()V".to_owned())),
            Some(&Reach::Instance)
        );
        let mut dir = Refs::default();
        dir.add_class(&out.classes["x/Dir"]).unwrap();
        assert!(dir.extended.contains("java/lang/Enum"));
        // values() reads both constants.
        let fields = &dir.fields["x/Dir"];
        assert!(fields.contains_key(&("NORTH".to_owned(), "Lx/Dir;".to_owned())));
        assert!(fields.contains_key(&("EAST".to_owned(), "Lx/Dir;".to_owned())));
    }

    #[test]
    fn stand_ins_keep_the_game_s_hierarchy_and_pass_constructor_arguments_up() {
        let mut api = Api::default();
        let mut class = |name: &str,
                         parent: &str,
                         access: u16,
                         interfaces: &[&str],
                         methods: &[(&str, &str, u16)]| {
            api.classes.insert(
                name.to_owned(),
                ApiClass {
                    parent: parent.to_owned(),
                    interfaces: interfaces.iter().map(|s| (*s).to_owned()).collect(),
                    access,
                    methods: methods
                        .iter()
                        .map(|(n, d, a)| (((*n).to_owned(), (*d).to_owned()), *a))
                        .collect(),
                    fields: Vec::new(),
                },
            );
        };
        class(
            "g/Behaviour",
            "java/lang/Object",
            0x0421,
            &[],
            &[
                ("codec", "()Lg/Codec;", 0x0009),
                ("<init>", "(Lg/Props;)V", 0x0001),
            ],
        );
        class(
            "g/Block",
            "g/Behaviour",
            0x0021,
            &["g/ItemLike"],
            &[("<init>", "(Lg/Props;)V", 0x0001)],
        );
        class(
            "g/Slab",
            "g/Block",
            0x0021,
            &[],
            &[("<init>", "(Lg/Props;)V", 0x0001)],
        );
        class("g/ItemLike", "java/lang/Object", 0x0601, &[], &[]);
        let mut refs = Refs::default();
        refs.declared.insert("my/Mod".into());
        refs.extended.insert("g/Slab".into());
        refs.parents
            .insert("my/Mod".into(), ("g/Slab".into(), Vec::new()));
        refs.classes
            .extend(["my/Mod", "g/Slab", "g/Props", "g/Codec"].map(String::from));
        refs.methods
            .entry("my/Mod".into())
            .or_default()
            .insert(("codec".into(), "()Lg/Codec;".into()), Reach::Instance);
        refs.methods
            .entry("g/Slab".into())
            .or_default()
            .insert(("<init>".into(), "(Lg/Props;)V".into()), Reach::Instance);
        let out = run(&refs, &api);
        let names: Vec<&str> = out.classes.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "g/Behaviour",
                "g/Block",
                "g/Codec",
                "g/ItemLike",
                "g/Props",
                "g/Slab"
            ]
        );
        let mut slab = Refs::default();
        slab.add_class(&out.classes["g/Slab"]).unwrap();
        assert!(slab.extended.contains("g/Block"));
        // The constructor hands its properties to Block's, which hands them to Behaviour's.
        assert!(
            slab.methods["g/Block"].contains_key(&("<init>".to_owned(), "(Lg/Props;)V".to_owned()))
        );
        let mut block = Refs::default();
        block.add_class(&out.classes["g/Block"]).unwrap();
        assert!(block.interfaces.contains("g/ItemLike"));
        assert!(
            block.methods["g/Behaviour"]
                .contains_key(&("<init>".to_owned(), "(Lg/Props;)V".to_owned()))
        );
        // The static method the mod reached through its own class is declared where the API has it.
        let mut behaviour = Refs::default();
        behaviour.add_class(&out.classes["g/Behaviour"]).unwrap();
        assert!(behaviour.classes.contains("g/Codec"));
    }

    #[test]
    fn horizon_s_own_classes_get_what_the_mod_names_and_keep_their_own_initialiser() {
        let out = run(&refs(), &Api::default());
        let mut p = Patch::default();
        p.interfaces.insert("x/Handler".into());
        p.methods
            .insert(("extra".into(), "()I".into()), Reach::Instance);
        p.fields
            .insert(("GLASS".into(), "Lx/Server;".into()), Reach::Static);
        let makeable = BTreeSet::from(["x/Server".to_owned()]);
        // x/Colors already has a static initialiser (it makes STONE).
        let patched = patch(&out.classes["x/Colors"], &p, &makeable).unwrap();
        let mut back = Refs::default();
        back.add_class(&patched).unwrap();
        assert!(back.interfaces.contains("x/Handler"));
        let fields = &back.fields["x/Colors"];
        assert!(fields.contains_key(&("GLASS".to_owned(), "Lx/Server;".to_owned())));
        assert!(fields.contains_key(&("STONE".to_owned(), "Lx/Server;".to_owned())));
        assert_eq!(
            back.methods["x/Colors"].get(&("horizon$clinit".to_owned(), "()V".to_owned())),
            Some(&Reach::Static)
        );
        // Patching again adds nothing twice.
        let again = patch(&patched, &p, &makeable).unwrap();
        let (_, a) = crate::api::read(&patched).unwrap();
        let (_, b) = crate::api::read(&again).unwrap();
        assert_eq!(a.methods.len(), b.methods.len());
        assert_eq!(a.fields.len(), b.fields.len());
        assert_eq!(a.interfaces, b.interfaces);
    }

    #[test]
    fn descriptors_name_their_classes_and_count_their_slots() {
        assert_eq!(
            descriptor_classes("(Lx/A;[Lx/B;IJ)Lx/C;"),
            ["x/A", "x/B", "x/C"]
        );
        assert_eq!(signature("(Lx/A;[Lx/B;IJ[[D)V"), (6, "V"));
        assert_eq!(signature("()Lx/C;"), (0, "Lx/C;"));
        assert_eq!(class_of("[[Lx/A;"), Some("x/A"));
        assert_eq!(class_of("[I"), None);
        assert_eq!(
            load_arguments("(IJLx/A;D)V", true),
            [0x15, 1, 0x16, 2, 0x19, 4, 0x18, 5]
        );
    }

    #[test]
    fn switches_are_stepped_over_whole() {
        // tableswitch at 1: pad 2, default, low 0, high 1, two offsets; then a return.
        let mut code = vec![0x03, 0xaa, 0, 0];
        for w in [0i32, 0, 1, 0, 0] {
            code.extend_from_slice(&w.to_be_bytes());
        }
        code.push(0xb1);
        assert_eq!(instruction_length(&code, 0).unwrap(), 1);
        assert_eq!(instruction_length(&code, 1).unwrap(), code.len() - 2);
    }
}
