//! Native subset of OptiFine/ETF random entity texture selection.
//! Unsupported rule predicates disable that rule instead of changing its meaning.
use crate::{Name, Properties};
use std::collections::{BTreeMap, BTreeSet};

const MAX_BYTES: usize = 256 * 1024;
const MAX_RULES: usize = 256;
const MAX_VARIANTS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error { TooLarge, BadRule(String), TooManyRules }

#[derive(Clone, Debug)]
pub struct Rule {
    pub id: u16,
    pub textures: Vec<u16>,
    pub weights: Vec<u32>,
    pub name: Option<Name>,
    pub biomes: Option<BTreeSet<String>>,
    pub heights: Option<Vec<(i32, i32)>>,
}
#[derive(Clone, Debug, Default)]
pub struct TextureRules {
    pub rules: Vec<Rule>,
    pub skipped: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
pub struct EntityContext<'a> {
    /// Persistently assigned by the game; must remain stable for one entity.
    pub seed: u64,
    pub name: Option<&'a str>,
    pub biome: Option<&'a str>,
    pub block_y: Option<i32>,
}

fn unsigned_list(s: &str) -> Result<Vec<u16>, ()> {
    let mut nums = Vec::new();
    for token in s.split_whitespace() {
        if let Some((start, end)) = token.split_once('-') {
            let a = start.parse::<u16>().map_err(|_| ())?;
            let b = end.parse::<u16>().map_err(|_| ())?;
            if a == 0 || b < a || usize::from(b - a) >= MAX_VARIANTS { return Err(()); }
            nums.extend(a..=b);
        } else {
            let n = token.parse::<u16>().map_err(|_| ())?;
            if n == 0 { return Err(()); }
            nums.push(n);
        }
        if nums.len() > MAX_VARIANTS { return Err(()); }
    }
    if nums.is_empty() { Err(()) } else { Ok(nums) }
}
fn weights(s: &str, size: usize) -> Result<Vec<u32>, ()> {
    let parsed = s.split_whitespace().map(str::parse::<u32>)
        .collect::<Result<Vec<_>,_>>().map_err(|_| ())?;
    if parsed.len() != size || parsed.iter().any(|x| *x == 0) { return Err(()); }
    let total: u64 = parsed.iter().map(|x| u64::from(*x)).sum();
    if total == 0 || total > 1_000_000_000 { return Err(()); }
    Ok(parsed)
}
fn heights(s: &str) -> Result<Vec<(i32, i32)>, ()> {
    let mut ranges = Vec::new();
    for t in s.split_whitespace() {
        // Negative heights are valid; split after the first digit/sign group.
        let (a, b) = if let Some(i) = t[1..].find('-').map(|i| i + 1) {
            (t[..i].parse::<i32>().map_err(|_| ())?, t[i+1..].parse::<i32>().map_err(|_| ())?)
        } else {
            let x = t.parse::<i32>().map_err(|_| ())?; (x, x)
        };
        if a > b { return Err(()); }
        ranges.push((a,b));
        if ranges.len() > 64 { return Err(()); }
    }
    if ranges.is_empty() { Err(()) } else { Ok(ranges) }
}
fn hash(seed: u64, salt: u16) -> u64 {
    // Stable integer mixer, not randomized Rust HashMap hashing.
    let mut x = seed ^ u64::from(salt).wrapping_mul(0x9e3779b97f4a7c15);
    x ^= x >> 30; x = x.wrapping_mul(0xbf58476d1ce4e5b9);
    x ^= x >> 27; x = x.wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

impl Rule {
    pub fn matches(&self, context: &EntityContext<'_>) -> bool {
        if let Some(name) = &self.name {
            if !context.name.is_some_and(|text| name.matches(text)) { return false; }
        }
        if let Some(biomes) = &self.biomes {
            if !context.biome.is_some_and(|b| biomes.contains(b)) { return false; }
        }
        if let Some(ranges) = &self.heights {
            if !context.block_y.is_some_and(|y| ranges.iter().any(|&(a,b)| (a..=b).contains(&y))) {
                return false;
            }
        }
        true
    }
    fn pick(&self, seed: u64) -> u16 {
        let total: u64 = self.weights.iter().map(|w| u64::from(*w)).sum();
        let mut index = hash(seed, self.id) % total;
        for (&texture, &weight) in self.textures.iter().zip(&self.weights) {
            if index < u64::from(weight) { return texture; }
            index -= u64::from(weight);
        }
        self.textures[0] // unreachable for valid weights
    }
}
impl TextureRules {
    /// Returns a texture suffix. Suffix 1 means the vanilla/base texture.
    /// Returns None when no supported rule applies.
    pub fn select(&self, context: EntityContext<'_>) -> Option<u16> {
        self.rules.iter().find(|rule| rule.matches(&context)).map(|r| r.pick(context.seed))
    }
}

/// Preserve Java properties syntax using the shared Horizon parser.
pub fn parse(src: &str) -> Result<TextureRules, Error> {
    if src.len() > MAX_BYTES { return Err(Error::TooLarge); }
    let props = Properties::parse(src);
    let mut groups: BTreeMap<u16,BTreeMap<String,String>> = BTreeMap::new();
    for (key,value) in &props.entries {
        if let Some((field, n)) = key.rsplit_once('.') {
            if let Ok(index) = n.parse::<u16>() {
                if index == 0 { continue; }
                groups.entry(index).or_default().insert(field.to_owned(), value.to_owned());
            }
        }
    }
    if groups.len() > MAX_RULES { return Err(Error::TooManyRules); }
    let mut result = TextureRules::default();
    for (id, raw) in groups {
        let Some(variants) = raw.get("skins").or_else(|| raw.get("textures")) else {
            continue; // a property about an absent rule is not a rule
        };
        // ETF supports many predicates; never silently ignore a predicate
        // that might otherwise cause a rule to match every entity.
        let allowed = ["skins", "textures", "weights", "name", "names", "biomes", "heights"];
        if let Some(field) = raw.keys().find(|field| !allowed.contains(&field.as_str())) {
            result.skipped.push(format!("rule {id}: unsupported predicate {field}"));
            continue;
        }
        let Ok(textures) = unsigned_list(variants) else {
            result.skipped.push(format!("rule {id}: invalid texture list"));
            continue;
        };
        let weights = match raw.get("weights") {
            Some(w) => match weights(w, textures.len()) {
                Ok(w) => w, Err(()) => {
                    result.skipped.push(format!("rule {id}: invalid weights")); continue;
                }
            },
            None => vec![1; textures.len()],
        };
        let name = match raw.get("name").or_else(|| raw.get("names")) {
            Some(s) => match Name::parse(s) {
                Some(matcher) => Some(matcher),
                None => { result.skipped.push(format!("rule {id}: invalid name")); continue; }
            },
            None => None,
        };
        let biomes = raw.get("biomes").map(|s| s.split_whitespace()
            .map(str::to_owned).collect::<BTreeSet<_>>());
        let heights = match raw.get("heights") {
            Some(s) => match heights(s) {
                Ok(h) => Some(h),
                Err(()) => { result.skipped.push(format!("rule {id}: invalid heights")); continue; }
            },
            None => None,
        };
        result.rules.push(Rule { id, textures, weights, name, biomes, heights });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx(seed: u64) -> EntityContext<'static> {
        EntityContext { seed, name: Some("Villager"), biome: Some("minecraft:plains"), block_y: Some(64) }
    }
    #[test] fn stable_selection_and_weights() {
        let rules = parse("skins.1=1 2 3\nweights.1=1 9 1").unwrap();
        assert_eq!(rules.rules[0].weights, [1,9,1]);
        assert_eq!(rules.select(ctx(7)), rules.select(ctx(7)));
    }
    #[test] fn rules_respect_predicates() {
        let rules = parse("skins.1=3\nname.1=Villager\nbiomes.1=minecraft:plains\nheights.1=60-70").unwrap();
        assert_eq!(rules.select(ctx(7)), Some(3));
        assert_eq!(rules.select(EntityContext { biome: Some("minecraft:desert"), ..ctx(7) }), None);
    }
    #[test] fn unknown_predicate_disables_rule() {
        let rules = parse("skins.1=2\nprofessions.1=farmer").unwrap();
        assert_eq!(rules.rules.len(), 0);
        assert_eq!(rules.skipped.len(), 1);
    }
    #[test] fn invalid_weight_disables_rule() {
        let rules = parse("skins.1=1 2\nweights.1=0 1").unwrap();
        assert_eq!(rules.select(ctx(1)), None);
    }
    #[test] fn negative_heights() {
        let h = heights("-64--32 -8 50-60").unwrap();
        assert_eq!(h, [(-64,-32),(-8,-8),(50,60)]);
    }
}
