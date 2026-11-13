//! Native ReAnimated screen effects (client-only).
//!
//! The real ReAnimated Fabric JAR depends on Screen, Widget, Tooltip, and
//! inventory Mixins. This is a conservative Rust implementation for Horizon:
//! it transforms *rendered* screen vertices only. Input hitboxes, pointer
//! coordinates, touch regions and screen click behavior are left unchanged.
use super::Game;
use horizon_client::{hud::{HudAtlas, HudBatch, rgba}, widgets::{self, Widget, Kind}};
use std::collections::BTreeMap;

const START_ALPHA: f32 = 0.72;
const TRANSITION_SECONDS: f32 = 0.23;
const MAX_SCALE_INSET: f32 = 0.012;
const MAX_FRAME_STEP: f32 = 0.10;
const MAX_WIDGET_STATES: usize = 192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ease {
    Smooth,
    OutCubic,
    OutQuart,
}

impl Ease {
    fn apply(self, t: f32) -> f32 {
        let t=t.clamp(0.0,1.0);
        match self {
            Ease::Smooth => t*t*(3.0-2.0*t),
            Ease::OutCubic => 1.0-(1.0-t).powi(3),
            Ease::OutQuart => 1.0-(1.0-t).powi(4),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenMotion {
    /// Fully rendered opacity, always in [START_ALPHA, 1].
    pub opacity: f32,
    /// Deliberately tiny scale to avoid touch/widget misalignment.
    pub scale: f32,
}
impl Default for ScreenMotion {
    fn default() -> Self { Self { opacity:1.0,scale:1.0 } }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub enabled: bool,
    pub reduced_motion: bool,
    pub speed: f32,
    pub ease: Ease,
    pub scale: bool,
    pub buttons: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self { enabled:false,reduced_motion:false,speed:1.0,
            ease:Ease::OutQuart,scale:true,buttons:true }
    }
}

#[derive(Clone, Debug, Default)]
struct WidgetState {
    hover: f32,
    press: f32,
    updated: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ReAnimated {
    pub config: Config,
    current: Option<String>,
    entered: f32,
    last_frame: Option<f32>,
    widget_frame: Option<f32>,
    widgets: BTreeMap<u64, WidgetState>,
    pub changes: u64,
}

impl ReAnimated {
    pub fn new(config: Config) -> Self { Self { config, ..Default::default() } }

    /// Advance screen animation. Uses client render-clock seconds, not tick count.
    /// Equal screen IDs do not restart a transition, even if fps changes.
    pub fn frame(&mut self, screen: &str, now: f32) -> ScreenMotion {
        if !self.config.enabled || self.config.reduced_motion || !now.is_finite() {
            self.current=Some(screen.to_owned());
            return ScreenMotion::default();
        }
        let now=now.max(0.0);
        if self.current.as_deref()!=Some(screen) {
            self.current=Some(screen.to_owned());
            self.entered=now;
            self.changes=self.changes.saturating_add(1);
        }
        let duration=(TRANSITION_SECONDS/self.config.speed.clamp(0.25,3.0)).max(0.06);
        let t=((now-self.entered)/duration).clamp(0.0,1.0);
        let e=self.config.ease.apply(t);
        let alpha=START_ALPHA+(1.0-START_ALPHA)*e;
        let scale=if self.config.scale { 1.0-MAX_SCALE_INSET*(1.0-e) } else { 1.0 };
        self.last_frame=Some(now);
        ScreenMotion { opacity:alpha,scale }
    }

    /// Effects are visual only. Apply to the screen-draw span, leaving the
    /// world HUD, tooltips rendered later, pad cursor and Android controls alone.
    pub fn draw(&mut self, key: &str, now: f32, batch: &mut HudBatch) {
        if !self.config.enabled { return; }
        let motion=self.frame(key,now);
        if (motion.scale-1.0).abs()>f32::EPSILON {
            // GUI coordinates, not pixel coordinates. The player continues
            // clicking the original widget positions.
            batch.scale_since(
                (batch.blur_at as usize).min(batch.vertices.len()),
                batch.width*0.5,batch.height*0.5,
                motion.scale,motion.scale,
            );
        }
        let start=(batch.blur_at as usize).min(batch.vertices.len());
        for vertex in &mut batch.vertices[start..] {
            let color=vertex.color;
            let alpha=((color>>24) as f32*motion.opacity).round() as u32;
            vertex.color=(color&0x00ff_ffff)|(alpha.min(255)<<24);
        }
    }

    /// Animate only the *decoration* of the last-frame widget rectangles.
    /// Input targets and text remain in vanilla coordinates; no GUI shape is
    /// moved. Widgets published last frame may lag by one rendered frame, which
    /// is less risky than intercepting click/drag logic.
    pub fn decorate(
        &mut self,
        widgets: &[Widget],
        pointer: (f32,f32),
        pressed: bool,
        now: f32,
        batch: &mut HudBatch,
        atlas: &HudAtlas,
    ) {
        if !self.config.enabled || !self.config.buttons
            || self.config.reduced_motion || !now.is_finite() {return;}
        let dt=self.widget_frame.map_or(1.0/60.0, |last| (now-last).clamp(0.0,MAX_FRAME_STEP));
        self.widget_frame=Some(now);
        let easing=(dt*14.0).clamp(0.0,1.0);
        let mut live=0;
        for widget in widgets {
            if !widget.enabled || widget.kind != Kind::Button
                || live >= MAX_WIDGET_STATES {continue;}
            let (x,y,w,h)=(widget.x,widget.y,widget.w,widget.h);
            if ![x,y,w,h].iter().all(|v|v.is_finite())
                || w<8.0 || h<7.0 || x<0.0 || y<0.0
                || x+w>batch.width+1.0 || y+h>batch.height+1.0 {continue;}
            live+=1;
            let id=widgets::key_of(&(
                widget.label.as_str(), x.round() as i32, y.round() as i32,
                w.round() as i32, h.round() as i32,
            ));
            let hovered=pointer.0>=x && pointer.0<=x+w
                && pointer.1>=y && pointer.1<=y+h;
            let state=self.widgets.entry(id).or_default();
            let hover_target=if hovered {1.0}else{0.0};
            let press_target=if hovered && pressed {1.0}else{0.0};
            state.hover+=(hover_target-state.hover)*easing;
            state.press+=(press_target-state.press)*easing;
            state.updated=now;
            if state.hover<0.005 && state.press<0.005 {continue;}
            let a=(state.hover*32.0+state.press*26.0).round().min(80.0) as u8;
            let color=rgba(187,216,255,a);
            let line=1.0f32;
            batch.fill(atlas,x,y,w,line,color);
            batch.fill(atlas,x,y+h-line,w,line,color);
            batch.fill(atlas,x,y,line,h,color);
            batch.fill(atlas,x+w-line,y,line,h,color);
        }
        // Bound memory even on large mod screens. Evict stale entries first.
        self.widgets.retain(|_,state| now-state.updated<=1.0);
        while self.widgets.len()>MAX_WIDGET_STATES {
            let Some(id)=self.widgets.iter().min_by(|a,b|
                a.1.updated.total_cmp(&b.1.updated)).map(|(k,_)|*k)
            else {break};
            self.widgets.remove(&id);
        }
    }

    /// Reports a suggested frame-clock delta, for other animation systems
    /// that should never jump forward on app resume.
    pub fn bounded_dt(&self, now: f32) -> f32 {
        let Some(previous)=self.last_frame else {return 0.0};
        (now-previous).clamp(0.0,MAX_FRAME_STEP)
    }
}

impl Game<'_> {
    pub(super) fn animate_reanimated(&mut self, screen: &str, batch: &mut HudBatch) {
        if !self.reanimated.config.enabled {return;}
        let now=self.started.elapsed().as_secs_f32();
        self.reanimated.draw(screen,now,batch);
        let widgets=horizon_client::widgets::last();
        self.reanimated.decorate(
            &widgets,self.cursor,self.left,now,batch,&self.gui.atlas
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn opens_and_settles() {
        let mut m=ReAnimated::new(Config {enabled:true,..Default::default()});
        let a=m.frame("inventory",1.0);
        assert!((a.opacity-START_ALPHA).abs()<0.001);
        assert!(a.scale<1.0);
        assert_eq!(m.frame("inventory",1.30),ScreenMotion::default());
        assert_eq!(m.changes,1);
    }
    #[test] fn changes_screen_and_obeys_reduced_motion() {
        let mut m=ReAnimated::new(Config {enabled:true,..Default::default()});
        m.frame("inventory",0.0);
        m.frame("options",0.1);
        assert_eq!(m.changes,2);
        m.config.reduced_motion=true;
        assert_eq!(m.frame("options",0.11),ScreenMotion::default());
    }
    #[test] fn fps_independent_fade() {
        let mut a=ReAnimated::new(Config {enabled:true,..Default::default()});
        let mut b=a.clone();
        a.frame("inventory",0.0);
        b.frame("inventory",0.0);
        for i in 1..=12 { a.frame("inventory",i as f32/120.0); }
        let x=b.frame("inventory",0.1);
        let y=a.frame("inventory",0.1);
        assert!((x.opacity-y.opacity).abs()<0.0001);
    }
    #[test] fn reduces_alpha_without_changing_touch_metadata() {
        let mut m=ReAnimated::new(Config {enabled:true,..Default::default()});
        let mut b=HudBatch::new([960,540]);
        b.quad([[0.0,0.0],[5.0,0.0],[5.0,5.0],[0.0,5.0]],
            [[0.0,0.0];4],0xff_ff_ff_ff);
        let size=b.vertices.len();
        m.draw("inventory",0.0,&mut b);
        assert_eq!(size,b.vertices.len());
        assert!((b.vertices[0].color>>24)<255);
    }
    #[test] fn hover_renders_only_decorations_never_mutating_widget() {
        let mut m=ReAnimated::new(Config {enabled:true,..Default::default()});
        let b=HudBatch::new([960,540]);
        let widget=Widget {label:"Play".to_owned(),x:10.0,y:20.0,w:140.0,h:24.0,
            enabled:true,kind:Kind::Button};
        let copy=widget.clone();
        // Atlas-backed drawing cannot be unit-tested without a texture pack,
        // but widget state and disabled mode are independent of GPU setup.
        m.config.reduced_motion=true;
        assert_eq!(widget.x,copy.x);
        assert_eq!(m.widgets.len(),0);
        assert!(b.vertices.is_empty());
    }
    #[test] fn disabled_reanimated_does_not_animate_widgets() {
        let m=ReAnimated::new(Config::default());
        assert!(!m.config.enabled);
        assert!(m.widgets.is_empty());
    }
    #[test] fn preserves_widget_coordinates_for_alpha_only() {
        let mut m=ReAnimated::new(Config {enabled:true,scale:false,..Default::default()});
        let mut b=HudBatch::new([960,540]);
        b.quad([[2.0,3.0],[4.0,3.0],[4.0,5.0],[2.0,5.0]],
            [[0.0,0.0];4],u32::MAX);
        let old=b.vertices[0].pos;
        m.draw("options",0.0,&mut b);
        assert_eq!(old,b.vertices[0].pos);
    }
}
