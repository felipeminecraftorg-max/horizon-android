//! Original pre-rendered 96x96 Jade console icon images.
//! Unlike the old procedural `icon_hit` outlines, these are PNG resources,
//! decoded once on first use, then uploaded to Horizon's GPU UI texture atlas.
//! Their alpha is preserved so per-frame materials may light/tint them.
//!
//! All source images are authored for Horizon; no Xbox trademark or asset is used.
use super::Icon;
use std::{io::Cursor, sync::OnceLock};

struct Sprite {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

const SOURCES: [(&str, &[u8]); 7] = [
    ("mods", include_bytes!("launcher_icons/jade/mods.png")),
    ("packs", include_bytes!("launcher_icons/jade/packs.png")),
    ("shaders", include_bytes!("launcher_icons/jade/shaders.png")),
    ("files", include_bytes!("launcher_icons/jade/files.png")),
    ("skins", include_bytes!("launcher_icons/jade/skins.png")),
    ("settings", include_bytes!("launcher_icons/jade/settings.png")),
    ("play", include_bytes!("launcher_icons/jade/play.png")),
];
static DECODED: OnceLock<[Option<Sprite>; 7]> = OnceLock::new();

fn decode(bytes: &[u8]) -> Option<Sprite> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let size = reader.output_buffer_size()?;
    // Asset dimensions are fixed; reject malformed or unexpectedly large files.
    if size > 128 * 128 * 4 { return None; }
    let mut pixels = vec![0; size];
    let info = reader.next_frame(&mut pixels).ok()?;
    if info.color_type != png::ColorType::Rgba
        || info.bit_depth != png::BitDepth::Eight
        || info.width != 96 || info.height != 96
    {
        return None;
    }
    pixels.truncate(info.buffer_size());
    Some(Sprite { width: info.width, height: info.height, pixels })
}

/// The key is prefixed with launcher-material so the atlas does not add a
/// rounded-square mask to a transparent pictogram.
pub(super) fn texture(icon: Icon) -> Option<(&'static str, u32, u32, &'static [u8])> {
    let (i, key) = match icon {
        Icon::ConsoleMods => (0, "launcher-material:jade-icon:mods"),
        Icon::ConsolePacks => (1, "launcher-material:jade-icon:packs"),
        Icon::ConsoleShaders => (2, "launcher-material:jade-icon:shaders"),
        Icon::ConsoleFiles => (3, "launcher-material:jade-icon:files"),
        Icon::ConsoleSkins => (4, "launcher-material:jade-icon:skins"),
        Icon::ConsoleSettings => (5, "launcher-material:jade-icon:settings"),
        Icon::ConsolePlay => (6, "launcher-material:jade-icon:play"),
        _ => return None,
    };
    let cache = DECODED.get_or_init(|| SOURCES.map(|(_, data)| decode(data)));
    let sprite = cache[i].as_ref()?;
    Some((key, sprite.width, sprite.height, &sprite.pixels))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_jade_icon_is_a_real_decodable_rgba_png() {
        for (name, bytes) in SOURCES {
            let s = decode(bytes).unwrap_or_else(|| panic!("{name} PNG is invalid"));
            assert_eq!((s.width, s.height), (96, 96));
            assert_eq!(s.pixels.len(), 96 * 96 * 4);
            let filled = s.pixels.chunks_exact(4).filter(|p| p[3] >= 128).count();
            assert!(filled > 120, "{name} has no recognizable opaque silhouette");
            let clear = s.pixels.chunks_exact(4).filter(|p| p[3] == 0).count();
            assert!(clear > 1000, "{name} has no transparent surroundings");
        }
    }
}
