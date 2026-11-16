//! Nexo's own application model. No Ui, rectangles, native-page capture or legacy
//! screen labels: only game/account/content data and handles for backend actions.
use super::{Launcher,nexo_ui::Binding,pages::Action};
use crate::catalog::{Page,FmModal};
use horizon_client::{launcher_nexo::{LayoutEdit,data_url},touch::{self,Control},content_files::Kind};
use serde_json::{Value,json};
use std::{collections::BTreeSet,time::Instant};

struct Registry {old:Vec<Binding>,items:Vec<Binding>,next:u64,scope:String,ranges:BTreeSet<u64>}
impl Registry {
 fn bind(&mut self,key:&str,action:Action)->u64 {
  let identity=format!("{}:{key}",self.scope);
  if let Some(b)=self.items.iter().find(|b|b.identity==identity&&b.action==action){return b.id;}
  let id=self.old.iter().find(|b|b.identity==identity&&b.action==action).map(|b|b.id).unwrap_or_else(||{let id=self.next;self.next+=1;id});
  self.items.push(Binding{id,action,identity});id
 }
 fn button(&mut self,key:&str,label:&str,action:Action)->Value {json!({"id":self.bind(key,action),"label":label,"kind":"button"})}
 fn toggle(&mut self,key:&str,label:&str,action:Action,on:bool)->Value {json!({"id":self.bind(key,action),"label":label,"kind":"toggle","value":on})}
 fn field(&mut self,key:&'static str,label:&str,value:&str,secret:bool)->Value {json!({"id":self.bind(key,Action::Field(key)),"label":label,"kind":if secret{"password"}else{"field"},"key":key,"value":if secret{""}else{value}})}
 fn range(&mut self,key:&str,label:&str,action:Action,fraction:f32,caption:String)->Value {let id=self.bind(key,action);self.ranges.insert(id);json!({"id":id,"label":label,"kind":"range","value":fraction.clamp(0.,1.),"caption":caption})}
 fn option(&mut self,key:&str,label:&str,caption:String,action:Action)->Value {json!({"id":self.bind(key,action),"label":label,"kind":"option","caption":caption})}
}
fn group(id:&str,title:&str,description:&str,controls:Vec<Value>)->Value {json!({"id":id,"title":title,"description":description,"controls":controls})}
fn kind_name(kind:Kind)->&'static str {match kind {Kind::Mods=>"mods",Kind::Packs=>"packs",Kind::Shaders=>"shaders",_=>"files"}}
impl Launcher {
 pub(super) fn publish_domain(&mut self) {
  if !crate::nexo::standalone()||!self.nexo.presented{return;}
  if !self.nexo.dirty&&self.nexo.captured.elapsed().as_millis()<100{return;}
  let scope=format!("{}:{}:{}:{}:{}:{}",self.catalog.page.name(),self.catalog.settings_category.title(),self.login.visible,self.catalog.project_modal.is_some(),self.catalog.fm_current_dir,self.nexo.editor.is_some());
  let mut r=Registry{old:std::mem::take(&mut self.nexo.bindings),items:vec![],next:self.nexo.next,scope,ranges:BTreeSet::new()};
  let view=if let Some(editor)=&self.nexo.editor {self.nexo_layout_model(editor)}else{match self.catalog.page {
   Page::Mods|Page::Packs|Page::Shaders=>self.nexo_catalog_model(&mut r),
   Page::Installed=>self.nexo_files_model(&mut r),Page::Skins=>self.nexo_wardrobe_model(&mut r),
   Page::Settings=>self.nexo_settings_model(&mut r),Page::Play=>json!({"type":"home"}),
  }};
  let modal=self.nexo_modal_model(&mut r);
  let install=self.install.lock().unwrap_or_else(|e|e.into_inner());
  let theme=self.catalog.config.theme.resolve();
  let payload=json!({"schema":2,"live":true,"product":"nexo","page":self.catalog.page.name(),"name":self.login.profile.as_deref().unwrap_or(&self.catalog.config.name),"profile":self.catalog.config.profile,
   "ready":horizon_client::official_install::ready(&self.home),"busy":self.worker.is_some()||self.catalog.busy()||self.session_worker.is_some(),"launching":self.launching,"view":view,"modal":modal,
   "status":self.error.as_deref().unwrap_or(&self.catalog.status),"install":{"stage":install.stage.label(),"done":install.done,"total":install.total},"download":{"progress":self.catalog.progress_fraction(),"stage":self.catalog.progress_stage(),"title":self.catalog.downloading_title},
   "sounds":self.catalog.config.theme.interface_sounds,"reducedMotion":self.catalog.config.disable_launcher_animations,
   "accent":format!("#{:02x}{:02x}{:02x}",theme.accent&255,(theme.accent>>8)&255,(theme.accent>>16)&255),"replies":self.nexo.replies});
  drop(install);
  self.nexo.bindings=r.items;self.nexo.next=r.next;self.nexo.domain_ranges=r.ranges;
  if let Ok(text)=serde_json::to_string(&payload){if text!=self.nexo.last{crate::nexo::publish(&text);self.nexo.last=text;}}
  self.nexo.replies.clear();self.nexo.dirty=false;self.nexo.captured=Instant::now();
 }
 fn nexo_catalog_model(&self,r:&mut Registry)->Value {
  let c=&self.catalog;let page=c.page;let kind=if page==Page::Mods{Kind::Mods}else if page==Page::Packs{Kind::Packs}else{Kind::Shaders};
  let local=c.local.iter().enumerate().filter(|(_,item)|item.kind==kind).map(|(i,item)|json!({"key":item.file,"title":item.title,"description":item.subtitle,"badge":item.badge,"size":crate::catalog::format_file_size(item.size_bytes),"enabled":item.enabled,"toggle":r.bind(&format!("local:{}",item.file),Action::ToggleLocal(i)),"remove":r.bind(&format!("remove:{}",item.file),Action::DeleteLocal(i)),"export":r.bind(&format!("export:{}",item.file),Action::ExportLocal(i))})).collect::<Vec<_>>();
  let projects=c.results.iter().enumerate().map(|(i,p)|json!({"key":p.id,"title":p.title,"description":p.summary,"author":p.author,"downloads":p.downloads,"support":p.support().label(),"installable":p.support().installable(),"reason":p.support().reason(),"image":c.icons.get(&p.id).and_then(|(w,h,pixels)|data_url(*w,*h,pixels)),"details":r.bind(&format!("project:{}",p.id),Action::OpenProject(i)),"install":r.bind(&format!("install:{}",p.id),Action::Install(i)),"installLabel":c.install_button(p)})).collect::<Vec<_>>();
  let bundled=if page==Page::Mods{horizon_client::mod_list::ANDROID_BUNDLED.iter().map(|(id,title,description)|json!({"key":id,"title":title,"description":description,"enabled":c.bundled_enabled.contains(*id),"details":r.bind(id,Action::OpenBundledProject(id)),"toggle":r.bind(&format!("bundled:{id}"),Action::ToggleBundled(id))})).collect::<Vec<_>>()}else{vec![]};
  let sources=if page==Page::Packs{vec![]}else{[crate::shader_catalog::Origin::Horizon,crate::shader_catalog::Origin::Modrinth].into_iter().map(|origin|json!({"label":origin.source().label(),"active":if page==Page::Mods{c.config.mod_source==origin}else{c.config.shader_source==origin},"id":r.bind(origin.source().label(),if page==Page::Mods{Action::ModSource(origin)}else{Action::ShaderSource(origin)})})).collect()};
  json!({"type":"catalog","kind":kind_name(kind),"query":c.query,"offset":c.offset,"local":local,"projects":projects,"bundled":bundled,"sources":sources,
   "searchField":r.bind("query",Action::Field("search")),"search":r.bind("search",Action::Search),"import":r.bind("import",Action::Import(kind)),"folder":r.bind("folder",Action::OpenFolder(kind)),"previous":r.bind("previous",Action::Previous),"next":r.bind("next",Action::Next),"pause":r.bind("pause",Action::PauseCatalogDownload),"cancel":r.bind("cancel",Action::CancelCatalogDownload)})
 }
 fn nexo_files_model(&self,r:&mut Registry)->Value {
  let c=&self.catalog;
  json!({"type":"files","path":c.fm_current_dir,"selected":c.fm_selected.len(),"canPaste":c.fm_clipboard.is_some(),"canBack":!c.fm_back_stack.is_empty(),
   "roots":crate::document_store::FM_ROOT_DIRS.iter().map(|(path,label)|json!({"label":label,"id":r.bind(path,Action::FmNavigateRoot(path))})).collect::<Vec<_>>(),
   "entries":c.fm_entries.iter().enumerate().map(|(i,f)|json!({"key":f.rel_path,"name":f.name,"directory":f.is_dir,"size":crate::catalog::format_file_size(f.size_bytes),"kind":f.kind_label,"selected":c.fm_selected.contains(&f.rel_path),"open":r.bind(&format!("open:{}",f.rel_path),Action::FmOpenEntry(i)),"select":r.bind(&format!("select:{}",f.rel_path),Action::FmToggleSelect(i)),"details":r.bind(&format!("details:{}",f.rel_path),Action::FmOpenDetails(i)),"rename":r.bind(&format!("rename:{}",f.rel_path),Action::FmOpenRename(i)),"export":r.bind(&format!("export:{}",f.rel_path),Action::FmExportSingle(i)),"remove":r.bind(&format!("remove:{}",f.rel_path),Action::FmRequestDeleteSingle(i))})).collect::<Vec<_>>(),
   "tools":vec![r.button("up","Pasta anterior",Action::FmUp),r.button("import","Importar",Action::FmImportHere),r.button("new","Nova pasta",Action::FmOpenNewFolder),r.button("all","Selecionar tudo",Action::FmSelectAll),r.button("none","Limpar seleção",Action::FmClearSelection),r.button("copy","Copiar",Action::FmCopy),r.button("cut","Mover",Action::FmCut),r.button("paste","Colar",Action::FmPaste),r.button("export","Compartilhar seleção",Action::FmExportSelected),r.button("delete","Excluir seleção",Action::FmRequestDeleteSelected)]})
 }
 fn nexo_wardrobe_model(&self,r:&mut Registry)->Value {
  let mut data=self.wardrobe.nexo_state();data["type"]=json!("wardrobe");
  if let Some(items)=data["items"].as_array_mut(){for item in items{let i=item["index"].as_u64().unwrap_or(0)as usize;item["choose"]=json!(r.bind(&format!("skin:{}",item["id"]),Action::WardrobeSelect(i)));}}
  if let Some(items)=data["cloudCapes"].as_array_mut(){for item in items{let i=item["index"].as_u64().unwrap_or(0)as usize;item["choose"]=json!(r.bind(&format!("cape:{i}"),Action::WardrobeActivateCape(i)));}}
  data["tools"]=json!({"skins":r.bind("skins",Action::WardrobeKind(false)),"capes":r.bind("capes",Action::WardrobeKind(true)),"import":r.bind("import",Action::WardrobeImport),"cloud":r.bind("cloud",Action::WardrobeUseCloud),"upload":r.bind("upload",Action::WardrobeUpload),"model":r.bind("model",Action::WardrobeModel),"hideCape":r.bind("hideCape",Action::WardrobeHideCape)});data
 }
 fn nexo_settings_model(&self,r:&mut Registry)->Value {
  let c=&self.catalog.config;let game=c.game_home(&self.home);let v=horizon_client::settings::Settings::load(&game.join("options.txt"));let t=touch::Settings::load_from(&game.join("horizon-touch.json"));
  let video=group("video","Imagem e desempenho","Resolução do jogo, distância e apresentação.",vec![
   r.option("renderer","Renderizador",c.renderer.label().into(),Action::Renderer),r.range("resolution","Resolução de renderização",Action::RenderScaleDelta(0),(c.render_scale.percent()as f32-25.)/75.,format!("{}%",c.render_scale.percent())),
   r.range("chunks","Distância de visão",Action::RenderDistDelta(0),(v.render_distance as f32-2.)/30.,format!("{} chunks",v.render_distance)),r.range("simulation","Simulação",Action::SimDistDelta(0),(v.simulation_distance as f32-5.)/27.,format!("{} chunks",v.simulation_distance)),r.range("fov","Campo de visão",Action::FovDelta(0.),(v.fov-30.)/80.,format!("{:.0}°",v.fov)),
   r.option("fps","Limite de quadros",format!("{} FPS",v.max_fps),Action::CycleMaxFps),r.option("quality","Qualidade gráfica",v.graphics_preset.clone(),Action::CycleGraphicsPreset),r.option("clouds","Nuvens",v.clouds.clone(),Action::CycleClouds),r.toggle("vsync","Sincronização vertical",Action::ToggleVsync,v.vsync),r.toggle("lighting","Iluminação suave",Action::ToggleSmoothLighting,v.smooth_lighting),r.option("refresh","Taxa da tela",c.device.display_label(),Action::DisplayHz),r.option("aspect","Proporção da imagem",c.screen.aspect.label().into(),Action::Aspect),r.option("rotation","Orientação",c.orientation.label().into(),Action::Orientation),r.toggle("immersive","Tela inteira",Action::Immersive,c.screen.immersive),r.toggle("cutout","Aproveitar recorte da tela",Action::Cutout,!c.screen.avoid_cutout)]);
  let controls=group("controls","Controles e gestos","Organize os comandos no editor Nexo.",vec![r.button("editor","Editar posição dos controles",Action::OpenControlEditor),r.option("presets","Layouts salvos",t.preset_name.clone(),Action::ControlProfileChooser),r.option("input","Modo de entrada",c.input_host.label().into(),Action::InputHost),
   r.range("touch","Sensibilidade do toque",Action::TouchSensDelta(0.),(t.sensitivity-0.1)/2.9,format!("{:.2}",t.sensitivity)),r.range("mouse","Sensibilidade do mouse",Action::MouseSensDelta(0.),(v.sensitivity-0.05)/0.95,format!("{:.2}",v.sensitivity)),r.range("buttonScale","Tamanho dos botões",Action::ButtonScaleDelta(0.),(t.button_scale-0.6),format!("{:.0}%",t.button_scale*100.)),r.toggle("left","Mão esquerda",Action::ToggleLeftHanded,t.left_handed),r.toggle("invert","Inverter olhar vertical",Action::ToggleInvertY,t.invert_y),r.toggle("split","Mira central",Action::ToggleSplitControls,t.split_controls),r.toggle("sprint","Corrida contínua",Action::ToggleSprintLatch,t.toggle_sprint),r.toggle("autoJump","Pulo automático",Action::ToggleAutoJump,t.auto_jump),r.toggle("gyro","Giroscópio",Action::ToggleGyro,t.gyroscope),r.range("gyroScale","Sensibilidade do giroscópio",Action::GyroSensDelta(0.),(t.gyro_sensitivity-0.1)/2.9,format!("{:.2}",t.gyro_sensitivity)),r.toggle("pointer","Cursor virtual",Action::ToggleVirtualMouseEnabled,t.virtual_mouse_enabled),r.range("pointerScale","Tamanho do cursor",Action::PointerSizeDelta(0.),(t.virtual_mouse_pointer_scale-0.5)/1.5,format!("{:.0}%",t.virtual_mouse_pointer_scale*100.)),r.toggle("haptics","Vibração do toque",Action::ToggleTouchVibrate,t.vibrate),r.toggle("uiHaptics","Vibração dos menus",Action::ToggleTouchUiVibrate,t.ui_vibrate),r.button("exportControls","Exportar layout",Action::ExportControlProfile),r.button("importControls","Importar layout",Action::ImportControlProfile)]);
  let system=group("system","Jogo e dispositivo","Perfil de jogo, inicialização e recursos.",vec![r.option("profile","Instância do jogo",format!("Perfil {}",c.profile),Action::Profile),r.field("name","Nome do jogador",&c.name,false),r.field("server","Servidor ao iniciar",&c.server,false),r.option("workers","Trabalhadores de chunks",c.worker_limit.to_string(),Action::WorkerLimit),r.option("cache","Cache reutilizável",format!("{} MiB",c.cache_budget_mib),Action::CacheBudget),r.toggle("hints","Indicadores de desempenho",Action::PerformanceHints,c.device.performance_hints),r.toggle("gpu","Perfil da GPU",Action::GpuProfile,c.device.gpu_profile),r.toggle("gameMotion","Animações do jogo",Action::ToggleGameAnimations,!c.disable_game_animations),r.button("worlds","Meus mundos",Action::WorldFiles)]);
  let experimental=group("advanced","Ajustes avançados","Opções existentes do motor. Algumas exigem reiniciar a partida.",vec![r.toggle("super","Priorizar desempenho",Action::ToggleSuperPerf,c.super_perf_mode),r.toggle("battery","Economizar energia",Action::ToggleBatteryPerf,c.battery_perf_mode),r.toggle("maxFps","Modo de quadros máximos",Action::ToggleMaxFpsMode,c.max_fps_mode),r.toggle("latency","Toque de baixa latência",Action::ToggleZeroLatencyTouch,c.zero_latency_touch),r.toggle("upload","Envio rápido de chunks",Action::ToggleFastChunkUpload,c.fast_chunk_upload),r.toggle("leaves","Oclusão experimental de folhas",Action::ToggleExperimentalLeaves,c.experimental_leaf_culling)]);
  let diagnostics=group("diagnostics","Diagnóstico","Acompanhe a sessão e compartilhe um relatório quando precisar.",vec![r.toggle("record","Gravar desempenho",Action::RecordPerformance,c.record_performance),r.button("performance","Compartilhar desempenho",Action::ExportPerformance),r.button("diagnostics","Exportar diagnóstico",Action::ExportDiagnostics),r.button("logs","Ver registros",Action::Logs)]);
  json!({"type":"settings","groups":[video,controls,system,experimental,diagnostics],"summary":crate::diagnostics::summary(&self.home)})
 }
 fn nexo_modal_model(&self,r:&mut Registry)->Value {
  if self.exit_confirmation{return json!({"type":"confirm","title":"Encerrar o Nexo?","description":"Seus ajustes já estão salvos.","actions":[r.button("continue","Continuar",Action::Dismiss),r.button("exit","Encerrar",Action::ConfirmExit)]});}
  if self.login.visible {
   let mut data=self.login.nexo_state();data["type"]=json!("account");
   data["actions"]=json!({"microsoft":r.bind("microsoft",Action::MicrosoftAccount),"ely":r.bind("ely",Action::ElySignIn),"offline":r.bind("offline",Action::OfflineAccount),"start":r.bind("start",Action::DeviceSignIn),"elySubmit":r.bind("elySubmit",Action::SubmitEly),"saved":r.bind("saved",Action::SavedAccounts),"add":r.bind("add",Action::AddAccount),"close":r.bind("close",Action::CloseLogin),"copy":r.bind("copy",Action::CopyLoginCode),"link":r.bind("link",Action::LoginLink),"cancel":r.bind("cancel",Action::CancelLogin),"signout":r.bind("signout",Action::SignOut),"selectOffline":r.bind("selectOffline",Action::SelectOffline)});
   data["fields"]=json!([r.field("ely_username","Usuário ou e-mail",&self.login.username,false),r.field("ely_password","Senha","",true),r.field("ely_otp","Código de verificação","",true),r.field("name","Nome do jogador",&self.catalog.config.name,false)]);
   if let Some(items)=data["accounts"].as_array_mut(){for item in items{let i=item["index"].as_u64().unwrap_or(0)as usize;item["choose"]=json!(r.bind(&format!("account:{}",item["name"]),Action::SelectAccount(i)));}}return data;
  }
  if let Some(p)=&self.catalog.project_modal {
   return json!({"type":"project","title":p.project.title,"description":p.description,"author":p.author,"license":p.license,"downloads":p.downloads,"categories":p.categories,"loading":p.loading,
    "versions":p.versions.iter().enumerate().map(|(i,v)|json!({"id":r.bind(&format!("version:{}",v.id),Action::SelectModalVersion(i)),"title":v.number,"date":v.published,"active":i==p.selected_version,"blocked":v.blocked})).collect::<Vec<_>>(),"screenshots":p.screenshots.iter().take(6).filter_map(|(_,w,h,rgba)|data_url(*w,*h,rgba)).collect::<Vec<_>>(),"install":r.bind("installProject",Action::InstallModalProject),"close":r.bind("closeProject",Action::CloseProjectModal)});
  }
  if let Some(chooser)=&self.chooser{return json!({"type":"choice","title":chooser.title,"description":chooser.help,"choices":chooser.choices.iter().enumerate().map(|(i,(title,desc,action,on))|json!({"id":r.bind(&format!("choice:{i}:{title}"),*action),"title":title,"description":desc,"active":on})).collect::<Vec<_>>(),"close":r.bind("closeChoice",Action::Dismiss)});}
  if let Some(modal)=&self.catalog.fm_modal{return match modal {
   FmModal::Rename{..}=>json!({"type":"form","title":"Renomear","field":r.field("fm_rename","Novo nome",&self.catalog.fm_input_text,false),"submit":r.bind("renameSubmit",Action::FmSubmitRename),"close":r.bind("closeFile",Action::FmCloseModal)}),
   FmModal::NewFolder=>json!({"type":"form","title":"Nova pasta","field":r.field("fm_new_folder","Nome da pasta",&self.catalog.fm_input_text,false),"submit":r.bind("folderSubmit",Action::FmSubmitNewFolder),"close":r.bind("closeFile",Action::FmCloseModal)}),
   FmModal::ConfirmDelete{paths}=>json!({"type":"confirm","title":"Excluir arquivos?","description":format!("{} item(ns) selecionado(s).",paths.len()),"actions":[r.button("cancelDelete","Cancelar",Action::FmCloseModal),r.button("deleteFiles","Excluir",Action::FmConfirmDelete)]}),
   FmModal::Details(f)=>json!({"type":"details","title":f.name,"description":f.rel_path,"size":crate::catalog::format_file_size(f.size_bytes),"modified":f.modified_label,"kind":f.kind_label,"preview":f.preview_lines,"close":r.bind("closeFile",Action::FmCloseModal)}),
  };}
  if self.status_details||self.diagnostics_modal_open{return json!({"type":"details","title":"Diagnóstico da sessão","description":self.error.as_deref().unwrap_or(&self.catalog.status),"preview":[crate::diagnostics::summary(&self.home)],"close":r.bind("closeStatus",Action::StatusDetails(false))});}
  Value::Null
 }
 fn nexo_layout_model(&self,s:&touch::Settings)->Value {
  json!({"type":"editor","name":s.preset_name,"mode":s.mode.index(),"leftHanded":s.left_handed,"style":s.control_style,"controls":Control::ALL.into_iter().enumerate().map(|(i,c)|json!({"index":i,"title":match c {Control::Move=>"Movimento",Control::Sprint=>"Correr",Control::Jump=>"Pular / voar",Control::Sneak=>"Agachar / descer",Control::Attack=>"Atacar / minerar",Control::Use=>"Usar / colocar",Control::Chat=>"Conversa",Control::Pause=>"Pausar",Control::Settings=>"Ajustes",Control::Inventory=>"Inventário",Control::Debug=>"Informações",Control::Perspective=>"Perspectiva",Control::Offhand=>"Segunda mão",Control::Shield=>"Escudo"},"custom":false,"visible":s.visible(c),"placement":s.placement(c)})).chain(s.custom_buttons.iter().enumerate().map(|(i,c)|json!({"index":i,"title":if c.custom_label.is_empty(){&c.key}else{&c.custom_label},"custom":true,"binding":c.key,"placement":c.layouts[s.mode.index()]}))).collect::<Vec<_>>()})
 }
 pub(super) fn nexo_edit_layout(&mut self,edit:LayoutEdit)->Result<(),String> {
  let Some(settings)=&mut self.nexo.editor else{return Err("Abra o editor Nexo primeiro".into());};
  match edit {
   LayoutEdit::Mode{mode}=>{settings.mode=*touch::Mode::ALL.get(mode as usize).ok_or("Modo de controle inválido")?;},
   LayoutEdit::Set{index,custom,placement}=>{let mut p:touch::Placement=serde_json::from_value(placement).map_err(|_|"Posição inválida")?;p.sanitize();let mode=settings.mode.index();if custom {let button=settings.custom_buttons.get_mut(index).ok_or("Botão não encontrado")?;button.layouts[mode]=p;}else{let control=*Control::ALL.get(index).ok_or("Controle não encontrado")?;if settings.left_handed{p.center[0]=1.-p.center[0];}let slot=settings.slot(control);settings.layouts[mode][slot]=p;settings.button_visibility[index]=if p.visible_in_game{1}else{2};}},
   LayoutEdit::Add{binding,label}=>{if !touch::CustomButton::valid_token(&binding)||label.chars().count()>48||settings.custom_buttons.len()>=64{return Err("Atalho inválido".into());}let mut b=touch::CustomButton::new(binding,settings.custom_buttons.len());b.custom_label=label;settings.custom_buttons.push(b);},
   LayoutEdit::Remove{index}=>{if index>=settings.custom_buttons.len(){return Err("Botão não encontrado".into());}settings.custom_buttons.remove(index);},
   LayoutEdit::Reset=>{*settings=touch::Settings::default_for_style(settings.control_style);},
   LayoutEdit::Cancel=>{self.nexo.editor=None;},
   LayoutEdit::Save=>{let game=self.catalog.config.game_home(&self.home);settings.sanitize();if settings.is_default_preset(){settings.save_new_preset(&game,"Meu Nexo","Layout personalizado no Nexo").map_err(|e|e.to_string())?;}else{settings.save_replace_preset(&game).map_err(|e|e.to_string())?;}settings.save_to(&game.join("horizon-touch.json")).map_err(|e|e.to_string())?;self.nexo.editor=None;self.catalog.status="Layout Nexo salvo".into();},
  }self.nexo.dirty=true;Ok(())
 }
}
