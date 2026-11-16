//! Reuses the real launcher actions and views; there is no second installer,
//! account store, content catalog or filesystem implementation in JavaScript.
use super::{
    Launcher,
    pages::{self, Action},
};
use crate::catalog::{Page, SettingsCategory};
use horizon_client::{
    launcher_nexo::{self as view, Destination, Operation, Preset},
    ui_theme::{ExperiencePreset, ThemePreset},
};
use serde_json::{Value, json};
use std::time::Instant;
use winit::event_loop::ActiveEventLoop;

pub(super) struct Binding {
    pub id: u64,
    pub action: Action,
    pub identity: String,
}
pub(super) struct Host {
    pub presented: bool,
    pub dirty: bool,
    pub captured: Instant,
    pub bindings: Vec<Binding>,
    pub next: u64,
    pub last: String,
    pub replies: Vec<Value>,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            presented: false,
            dirty: true,
            captured: Instant::now(),
            bindings: vec![],
            next: 1,
            last: String::new(),
            replies: vec![],
        }
    }
}
fn inside(rect: [f32; 4], x: f32, y: f32) -> bool {
    view::contains(rect, x, y)
}
impl Launcher {
    pub(super) fn sync_nexo(&mut self) -> bool {
        let wanted = self.game.is_none()
            && self.control_editor.is_none()
            && self.renderer.as_ref().is_some_and(|r| !r.suspended)
            && self.catalog.config.theme.resolve().experience == ExperiencePreset::OldSchool;
        if wanted != self.nexo.presented {
            self.nexo.presented = wanted;
            self.nexo.dirty = true;
            self.nexo.last.clear();
            if !crate::nexo::show(wanted) && wanted {
                self.nexo.presented = false;
                self.catalog
                    .config
                    .theme
                    .apply_preset(ThemePreset::ObsidianViolet);
                let _ = self.catalog.config.save(&self.home);
                horizon_client::ui_theme::set_active_theme(self.catalog.config.theme.resolve());
            }
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        self.nexo.presented
    }
    pub(super) fn poll_nexo(&mut self, events: &ActiveEventLoop) {
        if !self.sync_nexo() {
            return;
        }
        for command in crate::nexo::take() {
            let scope = self.nexo_scope();
            let result = self.nexo_operation(command.op, events);
            if scope != self.nexo_scope() { self.nexo.bindings.clear(); }
            if let Err(error) = &result {
                self.catalog.status = error.clone();
            }
            self.nexo
                .replies
                .push(json!({"id":command.id,"error":result.err()}));
            self.nexo.dirty = true;
            if !self.sync_nexo() {
                break;
            }
        }
        if self.nexo.dirty
            && let Some(window) = &self.window
        {
            window.request_redraw();
        }
    }
    fn clear_nexo_overlays(&mut self) {
        self.chooser = None;
        self.login.hide();
        self.catalog.project_modal = None;
        self.catalog.modal_worker = None;
        self.catalog.fm_modal = None;
        self.status_details = false;
        self.diagnostics_modal_open = false;
        self.quick_panel_open = false;
        self.exit_confirmation = false;
        self.catalog.editing = None;
    }
    fn nexo_operation(&mut self, op: Operation, events: &ActiveEventLoop) -> Result<(), String> {
        match op {
            Operation::Ready => {}
            Operation::Page { page } => {
                self.clear_nexo_overlays();
                let action = match page {
                    Destination::Play => Action::Page(Page::Play),
                    Destination::Mods => Action::Page(Page::Mods),
                    Destination::Packs => Action::Page(Page::Packs),
                    Destination::Shaders => Action::Page(Page::Shaders),
                    Destination::Skins => Action::Page(Page::Skins),
                    Destination::Files => Action::Page(Page::Installed),
                    Destination::Settings => Action::SettingsCat(SettingsCategory::Overview),
                    Destination::Account => Action::Account,
                    Destination::Profile => Action::Profile,
                    Destination::Status => Action::StatusDetails(true),
                };
                self.dispatch_action(events, Some(action))
                    .map_err(|e| e.to_string())?;
            }
            Operation::Press { control } => {
                let index=self.nexo.bindings.iter().position(|b| b.id==control)
                    .ok_or("This view changed. Please choose the item again.")?;
                // Consume a press before dispatch: repeated taps cannot delete
                // the next row after a removal shifts native list indices.
                let binding=self.nexo.bindings.remove(index);
                self.dispatch_action(events, Some(binding.action)).map_err(|e|e.to_string())?;
            }
            Operation::Edit {
                control,
                text,
                finish,
            } => {
                let action = self
                    .nexo
                    .bindings
                    .iter()
                    .find(|b| b.id == control)
                    .map(|b| b.action)
                    .ok_or("This field is no longer open")?;
                let Action::Field(key) = action else {
                    return Err("This control is not an editable field".into());
                };
                self.catalog.editing = Some(key);
                let value = match key {
                    "ely_username" => &mut self.login.username,
                    "ely_password" => &mut self.login.password,
                    "ely_otp" => &mut self.login.otp,
                    "name" => &mut self.catalog.config.name,
                    "profile_name" => &mut self.catalog.config.custom_profile_name,
                    "profile_desc" => &mut self.catalog.config.custom_profile_desc,
                    "server" => &mut self.catalog.config.server,
                    "render_scale" => &mut self.catalog.render_scale_text,
                    "client_id" => &mut self.catalog.config.microsoft_client_id,
                    "theme_bg" => &mut self.catalog.config.theme.custom_bg_hex,
                    "theme_surface" => &mut self.catalog.config.theme.custom_surface_hex,
                    "theme_accent" => &mut self.catalog.config.theme.custom_accent_hex,
                    "theme_grad_start" => &mut self.catalog.config.theme.gradient_start_hex,
                    "theme_grad_mid" => &mut self.catalog.config.theme.gradient_mid_hex,
                    "theme_grad_end" => &mut self.catalog.config.theme.gradient_end_hex,
                    "installed_search" => &mut self.catalog.installed_query,
                    "fm_rename" | "fm_new_folder" => &mut self.catalog.fm_input_text,
                    "search" => &mut self.catalog.query,
                    _ => return Err("Unknown native field".into()),
                };
                if key.starts_with("ely_") && text.len() > 1024 {
                    return Err("Account field is too long".into());
                }
                *value = text;
                self.edit_text(horizon_client::osk::Output::Text(String::new()));
                if finish {
                    self.edit_text(horizon_client::osk::Output::Close);
                }
            }
            Operation::Range { control, value } => {
                let action = self
                    .nexo
                    .bindings
                    .iter()
                    .find(|b| b.id == control)
                    .map(|b| b.action)
                    .ok_or("This setting changed")?;
                if !pages::nexo_slider_actions()
                    .iter()
                    .any(|(_, a, _, _)| *a == action)
                {
                    return Err("Not a slider".into());
                }
                self.apply_slider_fraction(action, value);
            }
            Operation::Rotate { dx, dy } => {
                if self.catalog.page == Page::Skins {
                    self.wardrobe.camera.drag(dx, dy);
                    self.wardrobe.preview();
                }
            }
            Operation::Back => self.request_back(),
            Operation::Play => {
                self.clear_nexo_overlays();
                self.catalog.page = Page::Play;
                self.action(events).map_err(|e| e.to_string())?;
            }
            Operation::Preset { preset } => {
                let preset = match preset {
                    Preset::Standard => ThemePreset::ObsidianViolet,
                    Preset::OldSchool => ThemePreset::OldSchool,
                    Preset::LiquidGlass => ThemePreset::LiquidGlass,
                };
                self.dispatch_action(events, Some(Action::SelectThemePreset(preset)))
                    .map_err(|e| e.to_string())?;
            }
            Operation::Fallback => {
                self.catalog
                    .config
                    .theme
                    .apply_preset(ThemePreset::ObsidianViolet);
                self.catalog
                    .config
                    .save(&self.home)
                    .map_err(|e| e.to_string())?;
                horizon_client::ui_theme::set_active_theme(self.catalog.config.theme.resolve());
            }
        }
        Ok(())
    }
    fn nexo_scope(&self) -> String {
        let modal = self.login.visible || self.chooser.is_some() || self.catalog.project_modal.is_some()
            || self.catalog.fm_modal.is_some() || self.status_details || self.diagnostics_modal_open || self.exit_confirmation;
        format!(
            "{}:{}:{}:{}:{}:{}",
            self.catalog.page.name(),
            self.catalog.settings_category.title(),
            modal,
            self.login.offline,
            self.login.ely,
            self.catalog.fm_current_dir
        )
    }
    fn nexo_field_value(&self, key: &str) -> &str {
        match key {
            "name" => &self.catalog.config.name,
            "profile_name" => &self.catalog.config.custom_profile_name,
            "profile_desc" => &self.catalog.config.custom_profile_desc,
            "server" => &self.catalog.config.server,
            "render_scale" => &self.catalog.render_scale_text,
            "client_id" => &self.catalog.config.microsoft_client_id,
            "theme_bg" => &self.catalog.config.theme.custom_bg_hex,
            "theme_surface" => &self.catalog.config.theme.custom_surface_hex,
            "theme_accent" => &self.catalog.config.theme.custom_accent_hex,
            "theme_grad_start" => &self.catalog.config.theme.gradient_start_hex,
            "theme_grad_mid" => &self.catalog.config.theme.gradient_mid_hex,
            "theme_grad_end" => &self.catalog.config.theme.gradient_end_hex,
            "installed_search" => &self.catalog.installed_query,
            "fm_rename" | "fm_new_folder" => &self.catalog.fm_input_text,
            "search" => &self.catalog.query,
            "ely_username" => &self.login.username,
            // Password/OTP are entered in the active form, never reflected in snapshots.
            _ => "",
        }
    }
    pub(super) fn publish_nexo(&mut self, frame: view::Frame) {
        let modal = self.login.visible
            || self.chooser.is_some()
            || self.catalog.project_modal.is_some()
            || self.catalog.fm_modal.is_some()
            || self.status_details
            || self.diagnostics_modal_open
            || self.exit_confirmation;
        let scope = self.nexo_scope();
        let previous = std::mem::take(&mut self.nexo.bindings);
        let mut controls = Vec::new();
        let ranges = pages::nexo_slider_actions();
        let mut seen_ranges = Vec::new();
        for (rect, action) in self.targets.iter().copied() {
            if matches!(
                action,
                Action::Page(_) | Action::Play | Action::SkipIntro | Action::CloseQuickPanel
            ) {
                continue;
            }
            let field = frame.fields.iter().find(|f| view::same_rect(f.rect, rect));
            let button = frame
                .buttons
                .iter()
                .rev()
                .find(|b| view::same_rect(b.rect, rect));
            let mut lines: Vec<String> = frame
                .texts
                .iter()
                .filter(|t| inside(rect, t.x, t.y))
                .map(|t| t.text.clone())
                .filter(|s| !s.trim().is_empty())
                .collect();
            lines.dedup();
            let nearest = frame
                .texts
                .iter()
                .filter(|t| t.size >= 11. && (t.y - rect[1]).abs() < 35. && t.x < rect[0] + rect[2])
                .min_by(|a, b| {
                    ((a.y - rect[1]).abs() + (a.x - rect[0]).abs() * 0.02)
                        .total_cmp(&((b.y - rect[1]).abs() + (b.x - rect[0]).abs() * 0.02))
                });
            let mut label = field
                .map(|f| {
                    if f.label.is_empty() {
                        f.hint.clone()
                    } else {
                        f.label.clone()
                    }
                })
                .or_else(|| button.map(|b| b.label.clone()))
                .or_else(|| lines.first().cloned())
                .or_else(|| nearest.map(|t| t.text.clone()))
                .unwrap_or_else(|| "Select".into());
            let mut kind = if field.is_some() { "field" } else { "button" };
            let mut group = String::new();
            let mut identity = label.clone();
            match action {
                Action::Field(key) => {
                    identity = key.into();
                    kind = if key == "ely_password" {
                        "password"
                    } else {
                        "field"
                    };
                }
                Action::OpenProject(i) | Action::Install(i) => {
                    if let Some(p) = self.catalog.results.get(i) {
                        group = format!("project:{i}");
                        identity = format!("{}:{}", p.id, p.title);
                        if matches!(action, Action::OpenProject(_)) {
                            label = p.title.clone();
                            kind = "project";
                        }
                    }
                }
                Action::OpenBundledProject(id) | Action::ToggleBundled(id) => {
                    group = format!("bundled:{id}");
                    identity = id.into();
                    if matches!(action, Action::OpenBundledProject(_)) {
                        kind = "project";
                    }
                }
                Action::ToggleLocal(i) | Action::DeleteLocal(i) => {
                    if let Some(item) = self.catalog.local.get(i) {
                        group = format!("local:{i}");
                        identity = item.file.clone();
                        if matches!(action, Action::ToggleLocal(_)) {
                            label = item.title.clone();
                            kind = "toggle";
                        }
                    }
                }
                _ => {}
            }
            let switch = frame
                .switches
                .iter()
                .find(|(r, _)| inside(rect, r[0] + r[2] * 0.5, r[1] + r[3] * 0.5));
            if switch.is_some() && kind == "button" {
                kind = "toggle";
                if let Some(t) = nearest {
                    if label == "On" || label == "Off" {
                        label = t.text.clone();
                    }
                }
            }
            let range = ranges.iter().find(|(_, a, _, _)| *a == action);
            let mut fraction = None;
            if let Some((r, _, _, _)) = range {
                if seen_ranges.contains(&action) {
                    continue;
                }
                seen_ranges.push(action);
                kind = "range";
                fraction = frame
                    .ranges
                    .iter()
                    .find(|(f, _)| view::same_rect(*f, *r))
                    .map(|(_, v)| *v);
            }
            let identity = format!("{scope}:{identity}");
            let id = previous
                .iter()
                .find(|b| b.action == action && b.identity == identity)
                .map(|b| b.id)
                .unwrap_or_else(|| {
                    let n = self.nexo.next;
                    self.nexo.next += 1;
                    n
                });
            if self.nexo.bindings.iter().any(|b| b.action == action && b.identity == identity) {
                continue;
            }
            self.nexo.bindings.push(Binding {
                id,
                action,
                identity,
            });
            let image = frame
                .images
                .iter()
                .find(|i| {
                    inside(
                        rect,
                        i.rect[0] + i.rect[2] * 0.5,
                        i.rect[1] + i.rect[3] * 0.5,
                    )
                })
                .map(|i| i.src.clone());
            let checked = if let Action::ToggleLocal(i) = action {
                self.catalog.local.get(i).map(|i| i.enabled)
            } else {
                switch.map(|(_, on)| *on)
            };
            let color = match action {
                Action::SelectAccentSwatch(rgb) | Action::SelectBgSwatch(rgb)
                | Action::SelectGradStartSwatch(rgb) | Action::SelectGradEndSwatch(rgb) => Some(format!("#{rgb:06x}")),
                _ => None,
            };
            if let Some(color)=&color { label=color.clone(); }
            controls.push(json!({"id":id,"kind":kind,"label":label,"lines":lines,"rect":rect,"group":group,
                "value":if let Action::Field(key)=action{self.nexo_field_value(key).to_owned()}else{String::new()},
                "field":if let Action::Field(key)=action{Some(key)}else{None},"icon":frame.icons.iter().find(|g|inside(rect,g.rect[0]+g.rect[2]*0.5,g.rect[1]+g.rect[3]*0.5)).map(|g|g.name),"color":color,"hint":field.map(|f|f.hint.clone()),"checked":checked,"selected":button.is_some_and(|b|b.selected),"fraction":fraction,"image":image}));
        }
        let texts: Vec<_> = frame
            .texts
            .iter()
            .filter(|t| {
                (modal || t.y >= 120.)
                    && !self.targets.iter().any(|(r, _)| inside(*r, t.x, t.y))
            })
            .collect();
        let title = if self.login.visible {
            "Sua conta"
        } else if self.chooser.is_some() {
            "Selecionar"
        } else if self.catalog.project_modal.is_some() {
            "Detalhes"
        } else if self.catalog.page == Page::Settings {
            self.catalog.settings_category.title()
        } else {
            self.catalog.page.name()
        };
        let theme = self.catalog.config.theme.resolve();
        let snapshot = json!({"schema":1,"live":true,"page":self.catalog.page.name(),"title":title,"modal":modal,
            "settingsCategory":self.catalog.settings_category.title(),"fileCanBack":!self.catalog.fm_back_stack.is_empty(),
            "name":self.login.profile.as_deref().unwrap_or(&self.catalog.config.name),"profile":self.catalog.config.profile,
            "launching":self.launching,"ready":horizon_client::official_install::ready(&self.home),"busy":self.worker.is_some()||self.catalog.busy()||self.launching||self.session_worker.is_some(),
            "status":self.error.as_deref().unwrap_or(&self.catalog.status),"texts":texts,"controls":controls,
            "skin":if self.catalog.page==Page::Skins{self.wardrobe.nexo_image()}else{None},
            "sounds":self.catalog.config.theme.interface_sounds,"reducedMotion":self.catalog.config.disable_launcher_animations,
            "accent":format!("#{:02x}{:02x}{:02x}",theme.accent&255,(theme.accent>>8)&255,(theme.accent>>16)&255),
            "replies":self.nexo.replies});
        let Ok(payload) = serde_json::to_string(&snapshot) else {
            return;
        };
        if payload != self.nexo.last {
            crate::nexo::publish(&payload);
            self.nexo.last = payload;
        }
        self.nexo.replies.clear();
        self.nexo.dirty = false;
        self.nexo.captured = Instant::now();
    }
}
