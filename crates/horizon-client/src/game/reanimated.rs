//! Native ReAnimated screen effects (client-only).
//!
//! The real ReAnimated Fabric JAR depends on Screen, Widget, Tooltip, and
//! inventory Mixins. This is a conservative Rust implementation for Horizon:
//! it transforms *rendered* screen vertices only. Input hitboxes, pointer
//! coordinates, touch regions and screen click behavior are left unchanged.
use horizon_client::hud::HudBatch;

const START_ALPHA: f32 = 0.72;
const TRANSITION_SECONDS: f32 = 0.23;
const MAX_SCALE_INSET: f32 = 0.012;
const MAX_FRAME_STEP: f32 = 0.10;

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
}
impl Default for Config {
    fn default() -> Self {
        Self { enabled:false,reduced_motion:false,speed:1.0,
            ease:Ease::OutQuart,scale:true }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ReAnimated {
    pub config: Config,
    current: Option<String>,
    entered: f32,
    last_frame: Option<f32>,
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

    /// Reports a suggested frame-clock delta, for other animation systems
    /// that should never jump forward on app resume.
    pub fn bounded_dt(&self, now: f32) -> f32 {
        let Some(previous)=self.last_frame else {return 0.0};
        (now-previous).clamp(0.0,MAX_FRAME_STEP)
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
