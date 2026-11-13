//! Old School-only XMB secondary-page compositor.
//!
//! These are *functional* spatial menus, not the retired dashboard cards:
//! the existing Catalog, imported files, installers and Settings actions are
//! merely the data/command layer. Liquid Glass retains its original UI tree.
//! All interaction rectangles are mapped to the existing Action dispatcher.
use super::*;
use crate::catalog::{InstalledFilter, InstalledSubTab, Page, SettingsCategory};
use horizon_client::content_files::Kind;
use horizon_client::ui_theme::{launcher_xmb::Cross, ExperiencePreset};

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
    let action_width=if trailing.is_some(){if w<390.{76.}else{106.}}else{0.};
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
        let r=[x+w-action_width,y+12.,action_width-4.,34.];
        xmb_text_action(ui,targets,r,text,act,selected);
        if let Some(hit)=ui.clip_rect(r){list_targets.push((hit,act));}
    }
}

fn xmb_heading(ui:&mut Ui<'_>, x:f32,y:f32,w:f32, breadcrumb:&str,title:&str) {
    ui.text(Face::Regular,10.,x+3.,y,ink(0x9CC3E4,158),breadcrumb);
    let fitted=ui.fit_text(Face::Medium,19.,title,(w-12.).max(60.));
    ui.text(Face::Medium,19.,x+3.,y+19.,ink(WHITE,250),&fitted);
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
        xmb_text_action(ui,&mut self.targets,
            [x,action_y,open_w,30.],"Abrir pasta",Action::OpenFolder(kind),false);
        xmb_text_action(ui,&mut self.targets,
            [x+open_w+5.,action_y,open_w,30.],"Importar",Action::Import(kind),false);
        if matches!(page,Page::Mods|Page::Shaders) {
            let hx=x+w-145.;
            if hx>x+open_w*2.+3. {
                use crate::shader_catalog::Origin;
                let current=if page==Page::Mods{self.catalog.config.mod_source}
                    else{self.catalog.config.shader_source};
                for (index,origin) in [Origin::Horizon,Origin::Modrinth]
                    .into_iter().enumerate(){
                    xmb_text_action(ui,&mut self.targets,
                        [hx+index as f32*74.,action_y,70.,30.],
                        origin.source().label(),
                        if page==Page::Mods{Action::ModSource(origin)}
                            else{Action::ShaderSource(origin)},
                        current==origin);
                }
            }
        }

        let list_top=action_y+39.;
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
            let r=[x+w-170.,y+38.,75.,18.];
            if y>=list_top && y+58.<=bottom {
                xmb_text_action(ui,&mut self.targets,r,"Exportar",
                    Action::ExportLocal(i),false);
                xmb_text_action(ui,&mut self.targets,
                    [x+w-88.,y+38.,81.,18.],"Excluir",
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
        let compact=w<450.;
        // A small command strip stays functional without replicating the
        // Material toolbar. Secondary file operations follow below.
        let nav_y=top;
        let main=[
            ("Voltar",Action::FmBack),("Avançar",Action::FmForward),
            ("Subir",Action::FmUp),("Importar",Action::FmImportHere),
            ("+ Pasta",Action::FmOpenNewFolder),
        ];
        let gap=3.;
        let cell=(w-gap*4.)/5.;
        for (i,(label,action)) in main.into_iter().enumerate(){
            xmb_text_action(ui,&mut self.targets,
                [x+i as f32*(cell+gap),nav_y,cell,32.],
                label,action,i==3);
        }

        // Profile-root selector: text-based named locations.
        let root_y=nav_y+38.;
        let roots=crate::document_store::FM_ROOT_DIRS;
        let max_roots_per_line=if compact{4}else{8};
        let root_h=30.;
        for (i,(rel,label)) in roots.iter().enumerate(){
            let col=i%max_roots_per_line;
            let row=i/max_roots_per_line;
            let n=max_roots_per_line as f32;
            let cw=(w-4.*(n-1.))/n;
            xmb_text_action(ui,&mut self.targets,
                [x+col as f32*(cw+4.),root_y+row as f32*root_h,cw,28.],
                label,Action::FmNavigateRoot(rel),
                self.catalog.fm_current_dir==*rel);
        }
        let rows=roots.len().div_ceil(max_roots_per_line);
        let command_y=root_y+rows as f32*root_h+4.;
        let cmds=[
            ("Tudo",Action::FmSelectAll),("Limpar",Action::FmClearSelection),
            ("Copiar",Action::FmCopy),("Mover",Action::FmCut),
            ("Colar",Action::FmPaste),("Renomear",Action::FmOpenRenameSelected),
            ("Exportar",Action::FmExportSelected),
            ("Excluir",Action::FmRequestDeleteSelected),
        ];
        let cmds_per_line=if compact{4}else{8};
        let cmd_lines=cmds.len().div_ceil(cmds_per_line);
        let width=(w-4.*(cmds_per_line as f32-1.))/cmds_per_line as f32;
        for (i,(label,action)) in cmds.into_iter().enumerate() {
            xmb_text_action(ui,&mut self.targets,
                [x+(i%cmds_per_line)as f32*(width+4.),
                    command_y+(i/cmds_per_line)as f32*29.,width,27.],
                label,action,false);
        }
        let crumbs_y=command_y+cmd_lines as f32*29.+2.;
        let path=if self.catalog.fm_current_dir.is_empty() {
            format!("PERFIL {} / RAIZ",self.catalog.config.profile+1)
        } else {
            format!("PERFIL {} / {}",self.catalog.config.profile+1,
                self.catalog.fm_current_dir)
        };
        let fitted=ui.fit_text(Face::Regular,11.,&path,(w-20.).max(20.));
        ui.text(Face::Regular,11.,x+5.,crumbs_y,ink(MUTED,164),&fitted);
        if let Some(r)=ui.clip_rect([x,crumbs_y,w,22.]) {
            self.targets.push((r,Action::FmBreadcrumb(0)));
        }

        let list_top=crumbs_y+27.;
        let bottom=h-53.;
        let row_h=59.;
        let max_scroll=(self.catalog.fm_entries.len() as f32*row_h
            -(bottom-list_top)).max(0.);
        self.catalog.scroll=self.catalog.scroll.clamp(0.,max_scroll);
        self.list_view=Some([list_top,bottom,max_scroll]);
        ui.set_clip(Some([x,list_top,w,(bottom-list_top).max(1.)]));
        for (i,entry) in self.catalog.fm_entries.iter().enumerate(){
            let y=list_top+i as f32*row_h-self.catalog.scroll;
            let icon=if entry.is_dir{Icon::XmbFiles}
                else if entry.name.ends_with(".hmod")||entry.name.ends_with(".jar"){
                    Icon::XmbMods
                }else if entry.name.ends_with(".zip"){Icon::XmbPacks}
                else if entry.name.ends_with(".hshader"){Icon::XmbShaders}
                else{Icon::FileText};
            let selected=self.catalog.fm_selected.contains(&entry.rel_path);
            let subtitle=if entry.is_dir {
                format!("PASTA · {} itens",entry.size_bytes)
            } else {
                format!("{} · {}",
                    entry.kind_label,
                    crate::catalog::format_file_size(entry.size_bytes))
            };
            xmb_entry(ui,&mut self.targets,&mut self.list_targets,
                x,y,w,&entry.name,&subtitle,icon,Action::FmOpenEntry(i),
                Some(("Detalhes",Action::FmOpenDetails(i))),
                selected,list_top,bottom);
            // A separate left-edge selection checkbox, consistent with XMB.
            let check=[x+1.,y+11.,37.,37.];
            if let Some(r)=ui.clip_rect(check){
                self.targets.push((r,Action::FmToggleSelect(i)));
                self.list_targets.push((r,Action::FmToggleSelect(i)));
            }
        }
        ui.set_clip(None);
        if self.catalog.fm_entries.is_empty(){
            ui.text(Face::Regular,12.5,x+4.,list_top+12.,
                ink(MUTED,181),"Esta pasta está vazia.");
        }
    }
}
