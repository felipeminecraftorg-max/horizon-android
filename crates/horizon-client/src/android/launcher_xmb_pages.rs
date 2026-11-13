//! Old School-only XMB secondary-page compositor.
//!
//! These are *functional* spatial menus, not the retired dashboard cards:
//! the existing Catalog, imported files, installers and Settings actions are
//! merely the data/command layer. Liquid Glass retains its original UI tree.
//! All interaction rectangles are mapped to the existing Action dispatcher.
use super::*;
use crate::catalog::{InstalledFilter, InstalledSubTab, Page, SettingsCategory};
use horizon_client::content_files::Kind;
use horizon_client::ui_theme::launcher_xmb::Cross;

const WHITE: u32 = 0xEAF2FF;
const MUTED: u32 = 0xA8C6DF;

fn ink(rgb: u32, a: u8) -> u32 { color::hex(rgb, a) }

fn xmb_text_action(
    ui: &mut Ui<'_>,
    targets: &mut Vec<([f32;4],Action)>,
    r: [f32;4], label: &str, action: Action, selected: bool,
) {
    let [x,y,w,h]=r;
    if selected {
        // Focus is a thin aurora underline, never a rounded card/capsule.
        ui.fill(x+3.,y+h-3.,(w-6.).max(1.),1.,
            ink(0xB6DCFF,132));
        ui.fill(x+3.,y+h-2.,(w-6.).max(1.),0.8,
            ink(0xE2F4FF,50));
    }
    let sz=if w<64. {10.5} else if w<100. {11.5} else {12.5};
    let text=ui.fit_text(Face::Regular,sz,label,(w-12.).max(10.));
    ui.text(Face::Regular,sz,x+6.,y+((h-sz*1.45)*0.5).max(0.),
        ink(if selected {WHITE}else{MUTED},if selected {245}else{187}),&text);
    if let Some(hit)=ui.clip_rect(r) {targets.push((hit,action));}
}

fn xmb_search(
    ui: &mut Ui<'_>, targets:&mut Vec<([f32;4],Action)>,
    r:[f32;4], value:&str, placeholder:&str,
    action:Action, editing:bool,
) {
    let [x,y,w,h]=r;
    ui.fill(x,y+h-3.,w,1.,ink(0xC5DFFF,if editing {208}else{86}));
    ui.icon(Icon::Search,x+2.,y+(h-22.)*0.5,20.,ink(MUTED,196));
    let label=if value.is_empty(){placeholder}else{value};
    let fitted=ui.fit_text(Face::Regular,13.,label,(w-43.).max(20.));
    ui.text(Face::Regular,13.,x+30.,y+(h-19.)*0.5,
        ink(if value.is_empty(){MUTED}else{WHITE},if value.is_empty(){143}else{245}),&fitted);
    if editing {
        let tw=ui.measure(Face::Regular,13.,&fitted);
        ui.fill((x+31.+tw).min(x+w-8.),y+8.,1.5,h-16.,
            ink(0xECF7FF,220));
    }
    if let Some(hit)=ui.clip_rect(r) {targets.push((hit,action));}
}

/// One strictly linear, non-Material list row. Button hitboxes are disjoint
/// from the row target; tap-to-open and secondary actions stay independent.
#[allow(clippy::too_many_arguments)]
fn xmb_entry(
    ui: &mut Ui<'_>, targets:&mut Vec<([f32;4],Action)>,
    list_targets:&mut Vec<([f32;4],Action)>,
    x:f32, y:f32, w:f32, title:&str, subtitle:&str,
    glyph:Icon, main:Action, trailing:Option<(&str,Action)>,
    selected:bool, top:f32, bottom:f32,
) {
    const RH:f32=58.;
    if y+RH<top || y>bottom {return;}
    // Reserve enough room for optional secondary actions on two distinct
    // baselines. No full-row overlay can steal taps from Export/Delete.
    let action_width=if trailing.is_some(){140.}else{0.};
    let left=x+4.;
    let title_x=x+51.;
    let text_w=(w-65.-action_width).max(40.);
    if selected{
        ui.fill(x+1.,y+9.,2.,RH-19.,ink(0xBBDEFF,217));
        ui.fill(x+5.,y+10.,w-12.,RH-18.,ink(0x6BA8D7,9));
    }
    ui.icon(glyph,x+10.,y+16.,27.,ink(if selected{WHITE}else{0xBDD8EA},
        if selected{252}else{184}));
    let fitted=ui.fit_text(Face::Medium,15.5,title,text_w);
    ui.text(Face::Medium,15.5,title_x,y+6.,
        ink(WHITE,if selected{249}else{223}),&fitted);
    let summary=ui.fit_text(Face::Regular,10.8,subtitle,text_w);
    ui.text(Face::Regular,10.8,title_x,y+32.,ink(MUTED,173),&summary);
    // A subtle no-border separation line, never a framed "card".
    ui.fill(x+51.,y+RH-1.,(w-55.).max(0.),0.7,ink(0x87B8D5,26));
    let hit=[left,y,(w-action_width-6.).max(30.),RH-1.];
    if let Some(r)=ui.clip_rect(hit){
        targets.push((r,main));
        list_targets.push((r,main));
    }
    if let Some((text,act))=trailing {
        let r=[x+w-action_width,y+5.,action_width-4.,28.];
        xmb_text_action(ui,targets,r,text,act,selected);
        if let Some(hit)=ui.clip_rect(r){list_targets.push((hit,act));}
    }
}

fn xmb_heading(ui:&mut Ui<'_>, x:f32,y:f32,w:f32, breadcrumb:&str,title:&str) {
    ui.text(Face::Regular,10.,x+3.,y,ink(0x9CC3E4,158),breadcrumb);
    let fitted=ui.fit_text(Face::Medium,19.,title,(w-12.).max(60.));
    ui.text(Face::Medium,19.,x+3.,y+19.,ink(WHITE,250),&fitted);
}

/// Unified unframed setting row for Video, Launcher, Controls, Experimental,
/// Diagnostics, account and advanced theme settings. The original command
/// dispatch is untouched; every control variant keeps its real action.
#[allow(clippy::too_many_arguments)]
pub(super) fn xmb_setting_row(
    ui:&mut Ui<'_>,targets:&mut Vec<([f32;4],Action)>,
    list_targets:&mut Vec<([f32;4],Action)>,
    x:f32,y:f32,w:f32,h:f32,
    title:&str,subtitle:&str,control:SettingControl<'_>
) {
    let is_group=matches!(control,SettingControl::SectionHeader(..));
    if is_group {
        if let SettingControl::SectionHeader(badge,icon)=control {
            ui.icon(icon,x+6.,y+17.,23.,ink(0xDCF0FF,202));
            let title=ui.fit_text(Face::Medium,15.5,title,(w-48.).max(35.));
            ui.text(Face::Medium,15.5,x+43.,y+10.,ink(WHITE,243),&title);
            let badge=ui.fit_text(Face::Regular,10.,badge,(w-48.).max(35.));
            ui.text(Face::Regular,10.,x+43.,y+33.,ink(MUTED,169),&badge);
        }
        return;
    }
    let action_w=if w < 395. {136.} else {196.};
    let text_w=(w-action_w-30.).max(32.);
    let title=ui.fit_text(Face::Medium,14.5,title,text_w);
    let summary=ui.fit_text(Face::Regular,10.7,subtitle,text_w);
    ui.text(Face::Medium,14.5,x+8.,y+7.,ink(WHITE,230),&title);
    ui.text(Face::Regular,10.7,x+8.,y+32.,ink(MUTED,155),&summary);
    ui.fill(x+9.,y+h-2.,w-18.,0.7,ink(0x9BC7E5,29));
    let ctrl_x=x+w-action_w+2.;
    match control {
        SettingControl::SectionHeader(..)=>{}
        SettingControl::Toggle(on,action)=>{
            let r=[ctrl_x,y+11.,action_w-9.,37.];
            ui.icon(if on{Icon::Check}else{Icon::Close},
                ctrl_x+1.,y+20.,16.,ink(if on{0xCCEBFA}else{MUTED},211));
            xmb_text_action(ui,targets,
                [ctrl_x+18.,r[1],r[2]-18.,r[3]],
                if on{"Ativado"}else{"Desativado"},action,on);
            if let Some(hit)=ui.clip_rect(r){list_targets.push((hit,action));}
        }
        SettingControl::Button(label,action,accent)=>{
            let r=[ctrl_x,y+10.,action_w-6.,38.];
            xmb_text_action(ui,targets,r,label,action,accent);
            if let Some(hit)=ui.clip_rect(r){list_targets.push((hit,action));}
        }
        SettingControl::TextField{label,value,hint,active,action}=>{
            let r=[ctrl_x,y+9.,action_w-6.,40.];
            xmb_search(ui,targets,r,value,
                if hint.is_empty(){label}else{hint},action,active);
            if let Some(hit)=ui.clip_rect(r){list_targets.push((hit,action));}
        }
        SettingControl::Stepper(value,minus,plus)=>{
            let middle=[ctrl_x+36.,y+11.,action_w-78.,36.];
            let value=ui.fit_text(Face::Regular,12.2,value,middle[2]-8.);
            ui.centered_fit(Face::Regular,12.2,middle,ink(WHITE,220),&value);
            let r_minus=[ctrl_x,y+11.,35.,36.];
            let r_plus=[ctrl_x+action_w-40.,y+11.,35.,36.];
            xmb_text_action(ui,targets,r_minus,"−",minus,false);
            xmb_text_action(ui,targets,r_plus,"+",plus,false);
            if let Some(hit)=ui.clip_rect(r_minus){list_targets.push((hit,minus));}
            if let Some(hit)=ui.clip_rect(r_plus){list_targets.push((hit,plus));}
        }
        SettingControl::Slider{value_label,fraction,minus_action,plus_action}=>{
            let r_minus=[ctrl_x,y+12.,32.,35.];
            let r_plus=[ctrl_x+action_w-37.,y+12.,32.,35.];
            let track_x=ctrl_x+36.;
            let track_w=(action_w-80.).max(22.);
            ui.fill(track_x,y+36.,track_w,2.,ink(0x94B6D5,90));
            ui.fill(track_x,y+36.,track_w*fraction.clamp(0.,1.),2.,
                ink(0xD9F3FF,229));
            let text=ui.fit_text(Face::Regular,10.3,value_label,track_w);
            ui.text(Face::Regular,10.3,track_x,y+12.,ink(WHITE,214),&text);
            xmb_text_action(ui,targets,r_minus,"−",minus_action,false);
            xmb_text_action(ui,targets,r_plus,"+",plus_action,false);
            let track=[track_x,y+23.,track_w,24.];
            if let Some(hit)=ui.clip_rect(track){
                SLIDER_TRACK_TARGETS.with(|tracks|
                    tracks.borrow_mut().push((hit,minus_action,track_x,track_w)));
                list_targets.push((hit,minus_action));
            }
            if let Some(hit)=ui.clip_rect(r_minus){list_targets.push((hit,minus_action));}
            if let Some(hit)=ui.clip_rect(r_plus){list_targets.push((hit,plus_action));}
        }
    }
}

impl Launcher {
    /// XMB project details replace the Modrinth Material popup. Every
    /// available version stays selectable and installs through the original
    /// catalog operation; descriptions and remote screenshots are preserved.
    pub(super) fn draw_xmb_project_modal(&mut self,ui:&mut Ui<'_>,w:f32,h:f32){
        let Some(install_label)=self.catalog.project_modal.as_ref().map(|modal|{
            let version=modal.versions.get(modal.selected_version)
                .map_or_else(||"mais recente".to_string(),|v|v.number.clone());
            if let Some(id)=modal.bundled_id {
                if self.catalog.bundled_enabled.contains(id){
                    "Desativar mod integrado".to_string()
                }else{
                    format!("Ativar mod ({version})")
                }
            }else if crate::catalog::source_only_mod(&modal.project){
                "Formato não suportado nesta versão Android".to_string()
            }else if let Some(state)=self.catalog.download_label(&modal.project){
                state
            }else{
                format!("Baixar {} ({version})",modal.project.kind.label())
            }
        })else{return;};
        let Some(modal)=self.catalog.project_modal.as_mut()else{return;};
        self.targets.clear();
        self.list_targets.clear();
        ui.fill(0.,0.,w,h,ink(0x040916,214));
        let x=if h>w{20.}else{(w*0.17).max(34.)};
        let width=(w-x-22.).min(690.);
        ui.text(Face::Regular,10.,x,18.,ink(0xAACBEB,181),
            "CONTEÚDO / INFORMAÇÕES");
        let title=ui.fit_text(Face::Medium,20.,&modal.project.title,
            width-60.);
        ui.text(Face::Medium,20.,x,39.,ink(WHITE,248),&title);
        let info=format!("{} · {} · {}",modal.author,
            modal.project.source.label(),modal.license);
        let meta=ui.fit_text(Face::Regular,10.7,&info,width-17.);
        ui.text(Face::Regular,10.7,x,76.,ink(MUTED,169),&meta);
        xmb_text_action(ui,&mut self.targets,
            [x+width-84.,14.,84.,39.],"Fechar",
            Action::CloseProjectModal,false);

        let top=107.;
        let bottom=h-62.;
        let desc_lines=wrap_text_lines(ui,Face::Regular,12.5,
            &modal.description,(width-12.).max(80.));
        let desc_height=desc_lines.len().max(1)as f32*19.;
        let screenshots=modal.screenshots.len().min(3);
        let gallery_h=if screenshots>0{118.}else{0.};
        let version_h=modal.versions.len().min(6)as f32*54.+32.;
        let total=desc_height+34.+gallery_h+version_h+24.;
        let max_scroll=(total-(bottom-top)).max(0.);
        modal.scroll=modal.scroll.clamp(0.,max_scroll);
        self.list_view=Some([top,bottom,max_scroll]);
        let mut row=top-modal.scroll;
        ui.set_clip(Some([x,top,width,(bottom-top).max(1.)]));
        ui.text(Face::Regular,11.,x+4.,row,ink(0xB8DDF5,183),
            "SOBRE O PROJETO");
        row+=26.;
        for text in &desc_lines{
            if row+19.>=top && row<=bottom{
                let value=ui.fit_text(Face::Regular,12.5,text,width-12.);
                ui.text(Face::Regular,12.5,x+4.,row,ink(WHITE,207),&value);
            }
            row+=19.;
        }
        row+=12.;
        if screenshots>0{
            ui.text(Face::Regular,11.,x+4.,row,ink(0xB7D9EF,174),
                "IMAGENS DO PROJETO");
            row+=22.;
            let gap=8.;
            let cell=(width-gap*(screenshots as f32-1.))
                /screenshots as f32;
            let cell_h=83.;
            for(i,(caption,sw,sh,rgba))in modal.screenshots
                .iter().take(3).enumerate(){
                let sx=x+i as f32*(cell+gap);
                if row+cell_h>=top && row<=bottom{
                    let key=format!("xmb-shot:{}:{i}",modal.project.id);
                    let _=ui.image(&key,*sw,*sh,rgba,sx,row,cell,63.);
                    let cap=ui.fit_text(Face::Regular,9.5,caption,cell);
                    ui.text(Face::Regular,9.5,sx,row+66.,
                        ink(MUTED,171),&cap);
                }
            }
            row+=98.;
        }
        ui.text(Face::Regular,11.,x+4.,row,ink(0xB7D9EF,184),
            "VERSÕES DISPONÍVEIS");
        row+=28.;
        if modal.versions.is_empty(){
            ui.text(Face::Regular,11.5,x+10.,row,
                ink(MUTED,168),"Verificando versões disponíveis...");
            row+=42.;
        }
        for (i,version) in modal.versions.iter().take(6).enumerate(){
            let active=i==modal.selected_version;
            if row+49.>=top && row<=bottom{
                if active{ui.fill(x+1.,row+7.,2.,35.,ink(0xD9F0FF,210));}
                let label=ui.fit_text(Face::Medium,14.,&version.number,
                    width-44.);
                ui.text(Face::Medium,14.,x+13.,row+5.,
                    ink(WHITE,if active{251}else{206}),&label);
                ui.text(Face::Regular,10.,x+13.,row+28.,
                    ink(MUTED,160),
                    if active{"VERSÃO SELECIONADA"}else{"TOCAR PARA SELECIONAR"});
                if let Some(hit)=ui.clip_rect([x,row,width,50.]){
                    self.targets.push((hit,Action::SelectModalVersion(i)));
                    self.list_targets.push((hit,Action::SelectModalVersion(i)));
                }
            }
            row+=54.;
        }
        ui.set_clip(None);
        // A single clear primary command in the bottom action lane.
        xmb_text_action(ui,&mut self.targets,
            [x,h-54.,(width-104.).max(135.),43.],
            &install_label,Action::InstallModalProject,true);
        xmb_text_action(ui,&mut self.targets,
            [x+width-97.,h-54.,96.,43.],
            "Voltar",Action::CloseProjectModal,false);
    }

    /// Contextual file dialogs are XMB text commands, with real validation,
    /// Android text entry and irreversible delete confirmation preserved.
    pub(super) fn draw_xmb_file_modal(
        &mut self,ui:&mut Ui<'_>,w:f32,h:f32,ime_top:Option<f32>,
    ){
        let Some(modal)=self.catalog.fm_modal.clone()else{return;};
        self.targets.clear();
        self.list_targets.clear();
        self.list_view=None;
        ui.fill(0.,0.,w,h,ink(0x030A15,206));
        let x=if h>w{22.}else{(w*0.20).max(33.)};
        let width=(w-x-25.).min(620.);
        let mut y=if h>w{(h*0.22).max(50.)}else{72.};
        if let Some(ime_y)=ime_top {
            y=y.min((ime_y-195.).max(8.));
        }
        ui.text(Face::Regular,10.,x,y,ink(0xBAD9F0,191),
            "ARQUIVOS / COMANDO");
        y+=25.;
        match modal {
            crate::catalog::FmModal::ConfirmDelete{paths}=>{
                ui.text(Face::Medium,19.,x,y,ink(WHITE,252),
                    "Confirmar exclusão");
                let detail=format!("{} item(ns) serão apagados permanentemente",
                    paths.len());
                let detail=ui.fit_text(Face::Regular,12.2,&detail,width);
                ui.text(Face::Regular,12.2,x,y+39.,ink(MUTED,188),&detail);
                let names=paths.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
                let names=ui.fit_text(Face::Regular,11.,&names,width);
                ui.text(Face::Regular,11.,x,y+68.,ink(WHITE,204),&names);
                xmb_text_action(ui,&mut self.targets,
                    [x,y+113.,width*0.47,43.],"Cancelar",
                    Action::FmCloseModal,false);
                xmb_text_action(ui,&mut self.targets,
                    [x+width*0.50,y+113.,width*0.49,43.],
                    "Excluir arquivos",Action::FmConfirmDelete,true);
            }
            crate::catalog::FmModal::Rename{target_rel}=>{
                ui.text(Face::Medium,19.,x,y,ink(WHITE,250),
                    "Renomear");
                let path=ui.fit_text(Face::Regular,11.,&target_rel,width);
                ui.text(Face::Regular,11.,x,y+33.,ink(MUTED,169),&path);
                xmb_search(ui,&mut self.targets,[x,y+67.,width,44.],
                    &self.catalog.fm_input_text,"Novo nome",
                    Action::Field("fm_rename"),
                    self.catalog.editing==Some("fm_rename"));
                xmb_text_action(ui,&mut self.targets,
                    [x,y+128.,width*0.47,40.],"Cancelar",
                    Action::FmCloseModal,false);
                xmb_text_action(ui,&mut self.targets,
                    [x+width*0.50,y+128.,width*0.49,40.],"Salvar",
                    Action::FmSubmitRename,true);
            }
            crate::catalog::FmModal::NewFolder=>{
                ui.text(Face::Medium,19.,x,y,ink(WHITE,250),
                    "Nova pasta");
                let path=ui.fit_text(Face::Regular,11.,
                    &self.catalog.fm_current_dir,width);
                ui.text(Face::Regular,11.,x,y+34.,ink(MUTED,176),&path);
                xmb_search(ui,&mut self.targets,[x,y+69.,width,44.],
                    &self.catalog.fm_input_text,"Nome da pasta",
                    Action::Field("fm_new_folder"),
                    self.catalog.editing==Some("fm_new_folder"));
                xmb_text_action(ui,&mut self.targets,
                    [x,y+130.,width*0.47,40.],"Cancelar",
                    Action::FmCloseModal,false);
                xmb_text_action(ui,&mut self.targets,
                    [x+width*0.50,y+130.,width*0.49,40.],"Criar pasta",
                    Action::FmSubmitNewFolder,true);
            }
            crate::catalog::FmModal::Details(details)=>{
                let name=ui.fit_text(Face::Medium,18.5,&details.name,width);
                ui.text(Face::Medium,18.5,x,y,ink(WHITE,250),&name);
                let summary=format!("{} · {} · {}",details.kind_label,
                    crate::catalog::format_file_size(details.size_bytes),
                    details.modified_label);
                let summary=ui.fit_text(Face::Regular,11.,&summary,width);
                ui.text(Face::Regular,11.,x,y+33.,ink(MUTED,174),&summary);
                let text_top=y+69.;
                let bottom=h-70.;
                ui.set_clip(Some([x,text_top,width,(bottom-text_top).max(1.)]));
                for(i,line)in details.preview_lines.iter().enumerate(){
                    let yy=text_top+i as f32*16.;
                    if yy>=bottom{break;}
                    let fitted=ui.fit_text(Face::Mono,10.,line,width-10.);
                    ui.text(Face::Mono,10.,x+4.,yy,ink(WHITE,209),&fitted);
                }
                ui.set_clip(None);
                xmb_text_action(ui,&mut self.targets,
                    [x,h-56.,(width-10.).max(95.),43.],
                    "Fechar detalhes",Action::FmCloseModal,true);
            }
        }
    }
}

impl Launcher {
    pub(super) fn draw_xmb_catalog(
        &mut self, ui:&mut Ui<'_>, page:Page,
        x:f32,w:f32,h:f32,slide_y:f32
    ) {
        let c=Cross::from_screen(ui.w,ui.h);
        let heading_y=c.category_y+50.+slide_y;
        let (title,icon,kind)=match page {
            Page::Mods=>("Mods",Icon::XmbMods,Kind::Mods),
            Page::Shaders=>("Shaders",Icon::XmbShaders,Kind::Shaders),
            _=>("Pacotes de recursos",Icon::XmbPacks,Kind::Packs),
        };
        xmb_heading(ui,x,heading_y,w,"CONTEÚDO / EXPLORAR",title);

        let search_y=heading_y+51.;
        let search_w=(w-88.).max(90.);
        xmb_search(ui,&mut self.targets,[x,search_y,search_w,34.],
            &self.catalog.query,"Buscar no catálogo",
            Action::Field("search"),self.catalog.editing==Some("search"));
        xmb_text_action(ui,&mut self.targets,
            [x+search_w+4.,search_y,82.,34.],"Buscar",Action::Search,true);

        let action_y=search_y+41.;
        let open_w=if w<400.{104.}else{121.};
        // On portrait, source selection gets its own row instead of
        // disappearing when Import/Open Folder occupy the horizontal lane.
        let source_second_line=matches!(page,Page::Mods|Page::Shaders)
            && w < open_w*2.+165.;
        xmb_text_action(ui,&mut self.targets,
            [x,action_y,open_w,30.],"Abrir pasta",Action::OpenFolder(kind),false);
        xmb_text_action(ui,&mut self.targets,
            [x+open_w+5.,action_y,open_w,30.],"Importar",Action::Import(kind),false);
        if matches!(page,Page::Mods|Page::Shaders) {
            let hx=if source_second_line{x}else{x+w-145.};
            if source_second_line || hx>x+open_w*2.+3. {
                use crate::shader_catalog::Origin;
                let current=if page==Page::Mods{self.catalog.config.mod_source}
                    else{self.catalog.config.shader_source};
                for (index,origin) in [Origin::Horizon,Origin::Modrinth]
                    .into_iter().enumerate(){
                    xmb_text_action(ui,&mut self.targets,
                        [hx+index as f32*74.,
                            action_y+if source_second_line{34.}else{0.},70.,30.],
                        origin.source().label(),
                        if page==Page::Mods{Action::ModSource(origin)}
                            else{Action::ShaderSource(origin)},
                        current==origin);
                }
            }
        }

        let list_top=action_y+if source_second_line{73.}else{39.};
        let list_bottom=h-55.;
        let row_h=59.;

        let query=self.catalog.query.to_lowercase();
        let bundled:Vec<_>=if page==Page::Mods
            && self.catalog.config.mod_source==crate::shader_catalog::Origin::Horizon {
            horizon_client::mod_list::ANDROID_BUNDLED.into_iter()
                .filter(|(id,title,_)|id.contains(&query)
                    || title.to_lowercase().contains(&query)).collect()
        } else {Vec::new()};
        let remote:Vec<_>=self.catalog.results.iter().enumerate().filter(|(_,p)|{
            !(page==Page::Mods && p.source==crate::browser::SourceId::Horizon
                && horizon_client::mod_list::ANDROID_BUNDLED.iter()
                    .any(|(id,_,_)| *id==p.id))
        }).collect();
        let n=bundled.len()+remote.len();
        let max_scroll=(n as f32*row_h-(list_bottom-list_top)).max(0.);
        self.catalog.scroll=self.catalog.scroll.clamp(0.,max_scroll);
        self.list_view=Some([list_top,list_bottom,max_scroll]);
        ui.set_clip(Some([x,list_top,w,(list_bottom-list_top).max(1.)]));

        for (i,(id,title,description)) in bundled.iter().enumerate(){
            let y=list_top+i as f32*row_h-self.catalog.scroll;
            let on=self.catalog.bundled_enabled.contains(*id);
            let desc=self.catalog.enriched_summaries.get(*id)
                .map_or(*description,String::as_str);
            xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                x,y,w,title,desc,icon,Action::OpenBundledProject(id),
                Some((if on{"Ativado"}else{"Instalar"},Action::ToggleBundled(id))),
                on,list_top,list_bottom);
        }
        for (j,(i,p)) in remote.into_iter().enumerate(){
            let y=list_top+(j+bundled.len()) as f32*row_h-self.catalog.scroll;
            let downloaded=self.catalog.download_label(p);
            let detail=self.catalog.enriched_summaries.get(&p.id)
                .map_or(p.summary.as_str(),String::as_str);
            let trailing=if downloaded.is_some() {
                Some(("Ver",Action::OpenProject(i)))
            } else {
                Some(("Instalar",Action::Install(i)))
            };
            xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                x,y,w,&p.title,detail,icon,Action::OpenProject(i),trailing,
                downloaded.is_some(),list_top,list_bottom);
        }
        ui.set_clip(None);
        if n==0{
            let status=self.error.as_deref().unwrap_or(&self.catalog.status);
            let fitted=ui.fit_text(Face::Regular,13.,status,w-20.);
            ui.text(Face::Regular,13.,x+4.,list_top+20.,
                ink(MUTED,182),&fitted);
        }

        // Paging is real, not a decorative tab; it uses Catalog::offset.
        if list_bottom-list_top>50. {
            let nav_y=h-49.;
            xmb_text_action(ui,&mut self.targets,
                [x,nav_y,95.,36.],"Anterior",Action::Previous,false);
            xmb_text_action(ui,&mut self.targets,
                [x+101.,nav_y,95.,36.],"Próxima",Action::Next,false);
        }
    }

    pub(super) fn draw_xmb_settings_overview(
        &mut self,ui:&mut Ui<'_>,x:f32,w:f32,h:f32,slide_y:f32,
    ){
        let c=Cross::from_screen(ui.w,ui.h);
        let y=c.category_y+54.+slide_y;
        xmb_heading(ui,x,y,w,"HORIZON / SISTEMA","Configurações");
        let entries:[(&str,&str,Icon,Action);6]=[
            ("Vídeo e desempenho","Renderização, resolução e FPS",Icon::Monitor,
                Action::SettingsCat(SettingsCategory::Video)),
            ("Personalização","Estilo, temas, perfis e preferências",Icon::XmbSystem,
                Action::SettingsCat(SettingsCategory::Launcher)),
            ("Controles","HUD, gestos e sensibilidade",Icon::Gamepad,
                Action::SettingsCat(SettingsCategory::Controls)),
            ("Experimental","Recursos em desenvolvimento",Icon::Flask,
                Action::SettingsCat(SettingsCategory::Experimental)),
            ("Diagnósticos","Logs, relatórios e informações",Icon::Terminal,
                Action::SettingsCat(SettingsCategory::Diagnostics)),
            ("Conta","Entrar e gerenciar contas",Icon::XmbIdentity,Action::Account),
        ];
        let top=y+59.;
        let bottom=h-51.;
        let row_h=59.;
        let max_scroll=(entries.len() as f32*row_h-(bottom-top)).max(0.);
        self.catalog.scroll=self.catalog.scroll.clamp(0.,max_scroll);
        self.list_view=Some([top,bottom,max_scroll]);
        ui.set_clip(Some([x,top,w,(bottom-top).max(1.)]));
        for (i,(title,desc,icon,action)) in entries.into_iter().enumerate(){
            xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                x,top+i as f32*row_h-self.catalog.scroll,w,
                title,desc,icon,action,None,false,top,bottom);
        }
        ui.set_clip(None);
    }

    pub(super) fn draw_xmb_installed(
        &mut self,ui:&mut Ui<'_>,x:f32,w:f32,h:f32,slide_y:f32,
    ){
        let c=Cross::from_screen(ui.w,ui.h);
        let y=c.category_y+54.+slide_y;
        let fm=self.catalog.installed_sub_tab==InstalledSubTab::FileManager;
        xmb_heading(ui,x,y,w,"HORIZON / ARQUIVOS",
            if fm{"Gerenciador de arquivos"}else{"Biblioteca instalada"});
        // Context tabs are text links anchored to the same horizontal XMB
        // line, not segmented pill/capsule controls.
        let tab_y=y+51.;
        xmb_text_action(ui,&mut self.targets,[x,tab_y,107.,32.],
            "Biblioteca",Action::SetInstalledSubTab(InstalledSubTab::Library),!fm);
        xmb_text_action(ui,&mut self.targets,[x+114.,tab_y,135.,32.],
            "Meus arquivos",Action::SetInstalledSubTab(InstalledSubTab::FileManager),fm);
        if fm {
            self.draw_xmb_file_manager(ui,x,w,h,tab_y+40.);
        } else {
            self.draw_xmb_library(ui,x,w,h,tab_y+39.);
        }
    }

    fn draw_xmb_library(&mut self,ui:&mut Ui<'_>,x:f32,w:f32,h:f32,top:f32){
        let filter_y=top;
        let filters=InstalledFilter::ALL;
        let filter_w=(w-7.*3.)/4.;
        for (i,filter) in filters.into_iter().enumerate(){
            xmb_text_action(ui,&mut self.targets,
                [x+i as f32*(filter_w+7.),filter_y,filter_w,30.],
                if filter==InstalledFilter::Packs{"Packs"}else{filter.label()},
                Action::SetInstalledFilter(filter),
                self.catalog.installed_filter==filter);
        }
        let tools_y=filter_y+39.;
        let search_w=(w-168.).max(85.);
        xmb_search(ui,&mut self.targets,[x,tools_y,search_w,35.],
            &self.catalog.installed_query,"Buscar instalados",
            Action::Field("installed_search"),
            self.catalog.editing==Some("installed_search"));
        xmb_text_action(ui,&mut self.targets,
            [x+search_w+4.,tools_y,79.,35.],"Ordenar",
            Action::CycleInstalledSort,false);
        let kind=match self.catalog.installed_filter {
            InstalledFilter::Packs=>Kind::Packs,
            InstalledFilter::Shaders=>Kind::Shaders,
            _=>Kind::Mods,
        };
        xmb_text_action(ui,&mut self.targets,
            [x+search_w+86.,tools_y,79.,35.],"Importar",
            Action::Import(kind),true);
        let list_top=tools_y+43.;
        let bottom=h-55.;
        let q=self.catalog.installed_query.trim().to_lowercase();
        let items:Vec<_>=self.catalog.local.iter().enumerate().filter(|(_,item)|{
            self.catalog.installed_filter.matches(item.kind)
                && (q.is_empty() || item.title.to_lowercase().contains(&q)
                    || item.file.to_lowercase().contains(&q)
                    || item.subtitle.to_lowercase().contains(&q))
        }).collect();
        let row_h=59.;
        let max_scroll=(items.len() as f32*row_h-(bottom-list_top)).max(0.);
        self.catalog.scroll=self.catalog.scroll.clamp(0.,max_scroll);
        self.list_view=Some([list_top,bottom,max_scroll]);
        ui.set_clip(Some([x,list_top,w,(bottom-list_top).max(1.)]));
        for (j,(i,item)) in items.into_iter().enumerate(){
            let y=list_top+j as f32*row_h-self.catalog.scroll;
            let icon=match item.kind {
                Kind::Mods=>Icon::XmbMods,Kind::Packs=>Icon::XmbPacks,
                Kind::Shaders=>Icon::XmbShaders,
            };
            let desc=format!("{} · {} · {}",
                item.badge,if item.enabled{"ATIVADO"}else{"DESATIVADO"},
                item.subtitle);
            xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                x,y,w,&item.title,&desc,icon,Action::ToggleLocal(i),
                Some((if item.enabled{"Desativar"}else{"Ativar"},
                    Action::ToggleLocal(i))),
                item.enabled,list_top,bottom);
            // Export/Delete are genuine secondary actions. Place them on a
            // distinct context ribbon reachable by touch, preserving their
            // production targets without overlaying the main action.
            let r=[x+w-140.,y+34.,66.,22.];
            if y>=list_top && y+58.<=bottom {
                xmb_text_action(ui,&mut self.targets,r,"Exportar",
                    Action::ExportLocal(i),false);
                xmb_text_action(ui,&mut self.targets,
                    [x+w-70.,y+34.,66.,22.],"Excluir",
                    Action::DeleteLocal(i),false);
            }
        }
        ui.set_clip(None);
        if self.catalog.local.is_empty(){
            ui.text(Face::Regular,12.5,x+3.,list_top+24.,
                ink(MUTED,180),"Sem complementos locais. Use Importar para começar.");
        }
    }

    fn draw_xmb_file_manager(
        &mut self,ui:&mut Ui<'_>,x:f32,w:f32,h:f32,top:f32,
    ){
        // Never display a dense 8+ button toolbar. A console-style menu
        // has one "location" list, one "commands" list and one file list;
        // they all use the same physical vertical space and retain actions.
        let compact=h<550.;
        let cy=top;
        let actions=[
            ("Arquivos",Action::XmbFilesPanel(0),self.xmb_files_panel==0),
            ("Pastas",Action::XmbFilesPanel(1),self.xmb_files_panel==1),
            ("Ações",Action::XmbFilesPanel(2),self.xmb_files_panel==2),
        ];
        let cw=(w-10.)/3.;
        for (i,(label,act,on)) in actions.into_iter().enumerate(){
            xmb_text_action(ui,&mut self.targets,
                [x+i as f32*(cw+5.),cy,cw,33.],label,act,on);
        }
        let list_top=cy+if compact{39.}else{47.};
        let bottom=h-50.;
        let row_h=58.;
        match self.xmb_files_panel {
            1=>{
                let roots=crate::document_store::FM_ROOT_DIRS;
                let max=(roots.len() as f32*row_h-(bottom-list_top)).max(0.);
                self.catalog.scroll=self.catalog.scroll.clamp(0.,max);
                self.list_view=Some([list_top,bottom,max]);
                ui.set_clip(Some([x,list_top,w,(bottom-list_top).max(1.)]));
                for (i,(rel,label)) in roots.iter().enumerate(){
                    let y=list_top+i as f32*row_h-self.catalog.scroll;
                    let active=self.catalog.fm_current_dir==*rel;
                    xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                        x,y,w,label,
                        if active{"LOCAL ATUAL"}else{"ABRIR DIRETÓRIO"},
                        Icon::XmbFiles,Action::FmNavigateRoot(rel),None,
                        active,list_top,bottom);
                }
                ui.set_clip(None);
            }
            2=>{
                let selected=self.catalog.fm_selected.len();
                let clip=self.catalog.fm_clipboard.as_ref().map_or(0,|(a,_)|a.len());
                let actions=[
                    ("Voltar","Diretório anterior",Icon::ArrowLeft,Action::FmBack),
                    ("Avançar","Diretório seguinte",Icon::ArrowRight,Action::FmForward),
                    ("Subir","Pasta superior",Icon::XmbFiles,Action::FmUp),
                    ("Importar","Adicionar arquivos do Android",Icon::XmbFiles,
                        Action::FmImportHere),
                    ("Nova pasta","Criar diretório",Icon::XmbFiles,Action::FmOpenNewFolder),
                    ("Selecionar tudo","Selecionar arquivos",Icon::Check,Action::FmSelectAll),
                    ("Limpar seleção","Desmarcar arquivos",Icon::Close,Action::FmClearSelection),
                    ("Copiar","Copiar arquivos selecionados",Icon::Copy,Action::FmCopy),
                    ("Recortar","Mover arquivos selecionados",Icon::ArrowRight,Action::FmCut),
                    ("Colar","Colar do clipboard",Icon::Copy,Action::FmPaste),
                    ("Renomear","Renomear item selecionado",Icon::Pencil,
                        Action::FmOpenRenameSelected),
                    ("Exportar","Exportar seleção",Icon::Share,Action::FmExportSelected),
                    ("Excluir","Excluir seleção com confirmação",Icon::Trash,
                        Action::FmRequestDeleteSelected),
                ];
                let max=(actions.len() as f32*row_h-(bottom-list_top)).max(0.);
                self.catalog.scroll=self.catalog.scroll.clamp(0.,max);
                self.list_view=Some([list_top,bottom,max]);
                ui.set_clip(Some([x,list_top,w,(bottom-list_top).max(1.)]));
                for (i,(title,sub,glyph,act)) in actions.into_iter().enumerate(){
                    let subtitle=if i==7||i==8||i==10||i==11||i==12{
                        if selected==0{"SEM ARQUIVOS SELECIONADOS"}else{sub}
                    }else if i==9 && clip==0 {
                        "ÁREA DE TRANSFERÊNCIA VAZIA"
                    }else{sub};
                    xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                        x,list_top+i as f32*row_h-self.catalog.scroll,w,
                        title,subtitle,glyph,act,None,false,list_top,bottom);
                }
                ui.set_clip(None);
            }
            _=>{
                let count=self.catalog.fm_entries.len();
                let max=(count as f32*row_h-(bottom-list_top)).max(0.);
                self.catalog.scroll=self.catalog.scroll.clamp(0.,max);
                self.list_view=Some([list_top,bottom,max]);
                ui.set_clip(Some([x,list_top,w,(bottom-list_top).max(1.)]));
                for (i,entry) in self.catalog.fm_entries.iter().enumerate(){
                    let y=list_top+i as f32*row_h-self.catalog.scroll;
                    let icon=if entry.is_dir{Icon::XmbFiles}
                        else if entry.name.ends_with(".hmod")
                            ||entry.name.ends_with(".jar"){Icon::XmbMods}
                        else if entry.name.ends_with(".zip"){Icon::XmbPacks}
                        else if entry.name.ends_with(".hshader"){Icon::XmbShaders}
                        else{Icon::FileText};
                    let selected=self.catalog.fm_selected.contains(&entry.rel_path);
                    let subtitle=if entry.is_dir {
                        format!("PASTA · {} itens",entry.size_bytes)
                    }else{
                        format!("{} · {}",entry.kind_label,
                            crate::catalog::format_file_size(entry.size_bytes))
                    };
                    xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                        x,y,w,&entry.name,&subtitle,icon,Action::FmOpenEntry(i),
                        Some(("Detalhes",Action::FmOpenDetails(i))),
                        selected,list_top,bottom);
                    // Selection is a separate left-edge touch surface. It
                    // cannot overlap Details on the right.
                    let mark=[x,y+10.,38.,36.];
                    if let Some(hit)=ui.clip_rect(mark){
                        self.targets.push((hit,Action::FmToggleSelect(i)));
                        self.list_targets.push((hit,Action::FmToggleSelect(i)));
                    }
                }
                ui.set_clip(None);
                let rel=if self.catalog.fm_current_dir.is_empty(){"PERFIL / RAIZ"}
                    else{"PERFIL / DIRETÓRIO"};
                ui.text(Face::Regular,9.5,x+4.,list_top-14.,ink(MUTED,154),rel);
                if count==0{
                    ui.text(Face::Regular,12.5,x+3.,list_top+14.,
                        ink(MUTED,188),"Nenhum arquivo nesta pasta.");
                }
            }
        }
    }

}
