//! Semantic launcher frame for the owner-requested Old School front end.
//! Capturing never exports account storage or arbitrary filesystem access: the
//! Android owner binds each visible control to an opaque, short-lived action ID.
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

#[derive(Clone, Default)]
pub struct Frame {
    pub texts: Vec<Text>,
    pub fields: Vec<Field>,
    pub buttons: Vec<Button>,
    pub switches: Vec<([f32; 4], bool)>,
    pub ranges: Vec<([f32; 4], f32)>,
    pub images: Vec<Picture>,
    pub icons: Vec<Glyph>,
}
#[derive(Clone, Serialize)]
pub struct Text {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub text: String,
}
#[derive(Clone)]
pub struct Field {
    pub rect: [f32; 4],
    pub label: String,
    pub value: String,
    pub hint: String,
}
#[derive(Clone)]
pub struct Button {
    pub rect: [f32; 4],
    pub label: String,
    pub selected: bool,
}
#[derive(Clone)]
pub struct Glyph {
    pub rect: [f32; 4],
    pub name: &'static str,
}
pub fn icon(rect: [f32; 4], name: &'static str) {
    edit(|f| f.icons.push(Glyph { rect, name }));
}
#[derive(Clone, Serialize)]
pub struct Picture {
    pub rect: [f32; 4],
    pub src: String,
}
thread_local! { static CAPTURE: RefCell<Option<Frame>> = const { RefCell::new(None) }; }
pub fn active() -> bool {
    CAPTURE.with(|f| f.borrow().is_some())
}
pub fn begin() {
    CAPTURE.with(|f| *f.borrow_mut() = Some(Frame::default()));
}
pub fn finish() -> Frame {
    CAPTURE.with(|f| f.borrow_mut().take().unwrap_or_default())
}
pub fn modal() {
    CAPTURE.with(|f| {
        if f.borrow().is_some() {
            *f.borrow_mut() = Some(Frame::default());
        }
    });
}
fn edit(f: impl FnOnce(&mut Frame)) {
    CAPTURE.with(|slot| {
        if let Some(frame) = slot.borrow_mut().as_mut() {
            f(frame);
        }
    });
}
pub fn text(x: f32, y: f32, size: f32, value: &str) {
    edit(|f| {
        if f.texts.len() < 4096 {
            f.texts.push(Text {
                x,
                y,
                size,
                text: value.chars().take(8192).collect(),
            });
        }
    });
}
pub fn field(rect: [f32; 4], label: &str, value: &str, hint: &str) {
    edit(|f| {
        f.fields.push(Field {
            rect,
            label: label.into(),
            value: value.into(),
            hint: hint.into(),
        })
    });
}
pub fn button(rect: [f32; 4], label: &str, selected: bool) {
    edit(|f| {
        f.buttons.push(Button {
            rect,
            label: label.into(),
            selected,
        })
    });
}
pub fn switch(rect: [f32; 4], value: bool) {
    edit(|f| f.switches.push((rect, value)));
}
pub fn range(rect: [f32; 4], value: f32) {
    edit(|f| f.ranges.push((rect, value.clamp(0., 1.))));
}
pub fn contains(rect: [f32; 4], x: f32, y: f32) -> bool {
    x >= rect[0] - 1.
        && y >= rect[1] - 1.
        && x <= rect[0] + rect[2] + 1.
        && y <= rect[1] + rect[3] + 1.
}
pub fn same_rect(a: [f32; 4], b: [f32; 4]) -> bool {
    a.into_iter().zip(b).all(|(a, b)| (a - b).abs() < 2.)
}

pub fn data_url(w: u32, h: u32, rgba: &[u8]) -> Option<String> {
    if w == 0 || h == 0 || w > 1024 || h > 1024 || rgba.len() != (w as usize) * (h as usize) * 4 {
        return None;
    }
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(rgba).ok()?;
    }
    Some(format!("data:image/png;base64,{}", encode64(&png)))
}
fn encode64(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for part in data.chunks(3) {
        let n = ((part[0] as u32) << 16)
            | ((part.get(1).copied().unwrap_or(0) as u32) << 8)
            | part.get(2).copied().unwrap_or(0) as u32;
        out.push(A[(n >> 18) as usize] as char);
        out.push(A[((n >> 12) & 63) as usize] as char);
        out.push(if part.len() > 1 {
            A[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if part.len() > 2 {
            A[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}
pub fn picture(rect: [f32; 4], w: u32, h: u32, rgba: &[u8]) {
    if !active() {
        return;
    }
    if let Some(src) = data_url(w, h, rgba) {
        edit(|f| {
            if f.images.len() < 48 {
                f.images.push(Picture { rect, src });
            }
        });
    }
}

#[derive(Debug, Deserialize)]
pub struct Command {
    pub id: u64,
    #[serde(flatten)]
    pub op: Operation,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Operation {
    Ready,
    Page {
        page: Destination,
    },
    Press {
        control: u64,
    },
    Edit {
        control: u64,
        text: String,
        #[serde(default)]
        finish: bool,
    },
    Range {
        control: u64,
        value: f32,
    },
    Rotate {
        dx: f32,
        dy: f32,
    },
    Back,
    Play,
    Preset {
        preset: Preset,
    },
    Layout {
        edit: LayoutEdit,
    },
    Accent {
        color: String,
    },
    Fallback,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case", deny_unknown_fields)]
pub enum LayoutEdit {
    Set {
        index: usize,
        #[serde(default)]
        custom: bool,
        placement: serde_json::Value,
    },
    Mode {
        mode: u8,
    },
    Add {
        binding: String,
        label: String,
    },
    Remove {
        index: usize,
    },
    Save,
    Cancel,
    Reset,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Destination {
    Play,
    Mods,
    Packs,
    Shaders,
    Skins,
    Files,
    Settings,
    Account,
    Profile,
    Status,
}
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Preset {
    Standard,
    OldSchool,
    LiquidGlass,
}
pub fn parse(input: &str) -> Result<Command, String> {
    if input.len() > 16384 {
        return Err("Launcher request is too large".into());
    }
    let value: serde_json::Value =
        serde_json::from_str(input).map_err(|_| "Unsupported launcher request")?;
    let object = value.as_object().ok_or("Expected a launcher request")?;
    let allowed: &[&str] = match object.get("op").and_then(serde_json::Value::as_str) {
        Some("ready" | "back" | "play" | "fallback") => &["id", "op"],
        Some("page") => &["id", "op", "page"],
        Some("press") => &["id", "op", "control"],
        Some("edit") => &["id", "op", "control", "text", "finish"],
        Some("range") => &["id", "op", "control", "value"],
        Some("rotate") => &["id", "op", "dx", "dy"],
        Some("preset") => &["id", "op", "preset"],
        Some("layout") => &["id", "op", "edit"],
        Some("accent") => &["id", "op", "color"],
        _ => return Err("Unsupported launcher action".into()),
    };
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unsupported launcher parameter".into());
    }
    if object.get("op").and_then(serde_json::Value::as_str) == Some("layout") {
        let edit = object
            .get("edit")
            .and_then(serde_json::Value::as_object)
            .ok_or("Missing layout command")?;
        let allowed: &[&str] = match edit.get("action").and_then(serde_json::Value::as_str) {
            Some("save" | "cancel" | "reset") => &["action"],
            Some("mode") => &["action", "mode"],
            Some("set") => &["action", "index", "custom", "placement"],
            Some("add") => &["action", "binding", "label"],
            Some("remove") => &["action", "index"],
            _ => return Err("Unsupported layout action".into()),
        };
        if edit.keys().any(|key| !allowed.contains(&key.as_str())) {
            return Err("Unsupported layout parameter".into());
        }
    }
    let cmd: Command = serde_json::from_value(value).map_err(|_| "Unsupported launcher request")?;
    if cmd.id == 0 {
        return Err("Missing request ID".into());
    }
    match &cmd.op {
        Operation::Layout {
            edit: LayoutEdit::Mode { mode },
        } if *mode > 2 => return Err("Invalid control mode".into()),
        Operation::Layout {
            edit: LayoutEdit::Set {
                index, placement, ..
            },
        } if *index >= 128 || !placement.is_object() => {
            return Err("Invalid control placement".into());
        }
        Operation::Layout {
            edit: LayoutEdit::Add { binding, label },
        } if binding.len() > 128
            || label.chars().count() > 48
            || binding.contains('\0')
            || label.contains('\0') =>
        {
            return Err("Invalid custom control".into());
        }
        Operation::Layout {
            edit: LayoutEdit::Remove { index },
        } if *index >= 64 => return Err("Invalid custom control index".into()),
        Operation::Accent { color }
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|c| c.is_ascii_hexdigit()) =>
        {
            return Err("Invalid accent color".into());
        }
        Operation::Edit { text, .. } if text.len() > 4096 || text.contains('\0') => {
            return Err("Text is too long".into());
        }
        Operation::Range { value, .. } if !value.is_finite() || !(0.0..=1.0).contains(value) => {
            return Err("Invalid range".into());
        }
        Operation::Rotate { dx, dy }
            if !dx.is_finite() || !dy.is_finite() || dx.abs() > 720. || dy.abs() > 720. =>
        {
            return Err("Invalid rotation".into());
        }
        _ => {}
    }
    Ok(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_typed_actions_and_bounded_values_cross_the_bridge() {
        assert!(parse(r#"{"id":1,"op":"page","page":"mods"}"#).is_ok());
        assert!(parse(r#"{"id":2,"op":"press","control":19}"#).is_ok());
        assert!(parse(r##"{"id":3,"op":"accent","color":"#7f50d0"}"##).is_ok());
        assert!(parse(r#"{"id":3,"op":"accent","color":"javascript:script"}"#).is_err());
        for bad in [
            r#"{"id":0,"op":"play"}"#,
            r#"{"id":1,"op":"delete","path":"/data/worlds"}"#,
            r#"{"id":1,"op":"range","control":2,"value":3}"#,
            r#"{"id":1,"op":"play","path":"/data"}"#,
            r#"{"id":1,"op":"preset","preset":"script"}"#,
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert!(
            parse(&format!(
                r#"{{"id":1,"op":"edit","control":2,"text":"{}"}}"#,
                "a".repeat(4097)
            ))
            .is_err()
        );
    }
    #[test]
    fn layout_commands_cannot_write_arbitrary_files_or_unbounded_controls() {
        assert!(parse(r#"{"id":1,"op":"layout","edit":{"action":"save"}}"#).is_ok());
        assert!(parse(r#"{"id":1,"op":"layout","edit":{"action":"mode","mode":2}}"#).is_ok());
        for bad in [
            r#"{"id":1,"op":"layout","edit":{"action":"save","path":"/data/world"}}"#,
            r#"{"id":1,"op":"layout","edit":{"action":"mode","mode":3}}"#,
            r#"{"id":1,"op":"layout","edit":{"action":"set","index":128,"placement":{}}}"#,
            r#"{"id":1,"op":"layout","edit":{"action":"set","index":0,"placement":"script"}}"#,
            r#"{"id":1,"op":"layout","edit":{"action":"remove","index":64}}"#,
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn modal_capture_excludes_background_values_and_is_scoped() {
        begin();
        text(0., 0., 14., "Background");
        field([0., 0., 100., 40.], "Account", "visible-name", "");
        modal();
        text(20., 20., 18., "Confirm deletion");
        button([20., 60., 120., 48.], "Cancel", false);
        let frame = finish();
        assert_eq!(frame.texts.len(), 1);
        assert!(frame.fields.is_empty());
        assert_eq!(frame.buttons[0].label, "Cancel");
        assert!(!active());
    }
    #[test]
    fn image_encoding_has_no_external_reference_and_preserves_alpha() {
        let src = data_url(1, 1, &[32, 64, 128, 0]).unwrap();
        assert!(src.starts_with("data:image/png;base64,iVBOR"));
        assert_eq!(encode64(b"a"), "YQ==");
        assert_eq!(encode64(b"ab"), "YWI=");
        assert_eq!(encode64(b"abc"), "YWJj");
        assert!(data_url(1, 1, &[0, 0, 0]).is_none());
    }
}
