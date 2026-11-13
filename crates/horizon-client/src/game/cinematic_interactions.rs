//! Cinematic Interactions native camera rig for workstation screens.
//!
//! The original Fabric mod contains 3-D workstation-specific scenes, menu
//! remapping, server sync, hands and camera Mixins. This initial native
//! implementation covers their presentation camera transitions without
//! changing inventory transactions, network packets, game state or input.
use super::{Game, Screen, stations};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Station {
    Crafting, Furnace, Brewing, Enchanting, Anvil, Grindstone,
    Smithing, Stonecutter, Cartography, Loom, Beacon, Other,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub fov: f32,
}
impl Pose {
    fn values(self) -> [f32;4] {[self.yaw,self.pitch,self.roll,self.fov]}
    fn from(values:[f32;4])->Self {
        Self {yaw:values[0],pitch:values[1],roll:values[2],fov:values[3]}
    }
}
impl Station {
    /// Small pose adjustments (degrees) around the existing camera output,
    /// preserving FirstPerson and the Camera Overhaul stack.
    pub fn target(self) -> Pose {
        match self {
            Self::Crafting => Pose{yaw:-0.25,pitch:1.2,roll:-0.45,fov:-2.0},
            Self::Furnace => Pose{yaw:0.30,pitch:0.7,roll:0.30,fov:-1.5},
            Self::Brewing => Pose{yaw:-0.45,pitch:0.9,roll:-0.25,fov:-1.5},
            Self::Enchanting => Pose{yaw:-0.5,pitch:1.3,roll:-0.6,fov:-2.2},
            Self::Anvil => Pose{yaw:0.35,pitch:1.8,roll:0.45,fov:-2.5},
            Self::Grindstone => Pose{yaw:0.4,pitch:1.1,roll:0.4,fov:-2.0},
            Self::Smithing => Pose{yaw:-0.25,pitch:1.0,roll:-0.35,fov:-2.0},
            Self::Stonecutter => Pose{yaw:0.2,pitch:0.8,roll:0.25,fov:-1.8},
            Self::Cartography => Pose{yaw:0.5,pitch:1.0,roll:0.5,fov:-1.6},
            Self::Loom => Pose{yaw:-0.30,pitch:0.7,roll:-0.45,fov:-1.7},
            Self::Beacon => Pose{yaw:0.0,pitch:1.6,roll:0.0,fov:-2.0},
            Self::Other => Pose::default(),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub enabled: bool,
    pub reduced_motion: bool,
    /// Duration multiplier; 1 is default, smaller is faster.
    pub speed: f32,
    /// Set to 0 to disable roll motion (accessible mode).
    pub roll_strength: f32,
}
impl Default for Config {
    fn default()->Self {Self{enabled:false,reduced_motion:false,speed:1.0,roll_strength:1.0}}
}
#[derive(Clone, Debug, Default)]
pub struct Rig {
    pub config: Config,
    pub station: Option<Station>,
    pub current: Pose,
    velocity: [f32;4],
    last_time: Option<f32>,
    pub transitions: u64,
    /// Time since last client-side workstation click; visual only.
    interaction_age: Option<f32>,
}
impl Rig {
    pub fn new(config:Config)->Self{Self{config,..Default::default()}}

    /// A workstation's existing menu action has already completed; add only
    /// a client-side camera accent. Never delay or re-send the action.
    pub fn pulse(&mut self) {
        if self.config.enabled && !self.config.reduced_motion && self.station.is_some() {
            self.interaction_age=Some(0.0);
        }
    }

    /// Critically damped second-order motion: integrates a bounded timestep
    /// and targets an exact rest pose on scene close or accessibility disable.
    /// The clock is absolute render time, never a 20Hz world tick.
    pub fn advance(&mut self, station:Option<Station>,seconds:f32)->Pose {
        if !self.config.enabled || self.config.reduced_motion || !seconds.is_finite() {
            self.station=None;
            self.velocity=[0.0;4];
            self.current=Pose::default();
            self.interaction_age=None;
            self.last_time=None;
            return self.current;
        }
        let dt=match self.last_time {
            Some(last)=>(seconds-last).clamp(0.0,0.080),
            None=>0.0,
        };
        self.last_time=Some(seconds);
        if let Some(age)=self.interaction_age {
            let age=age+dt;
            self.interaction_age=(age<0.9).then_some(age);
        }
        if station!=self.station {
            self.station=station;
            self.transitions=self.transitions.saturating_add(1);
        }
        let mut target=station.map(Station::target).unwrap_or_default().values();
        target[2]*=self.config.roll_strength.clamp(0.0,1.0);
        let mut values=self.current.values();
        // Exact solution for a critically damped spring with constant target.
        // Avoids numerical instability even if Android drops frames.
        let omega=16.0/self.config.speed.clamp(0.35,2.5);
        let decay=(-omega*dt).exp();
        for i in 0..4 {
            let displacement=values[i]-target[i];
            let change=(self.velocity[i]+omega*displacement)*dt;
            values[i]=target[i]+(displacement+change)*decay;
            self.velocity[i]=(self.velocity[i]-omega*change)*decay;
            if !values[i].is_finite()||!self.velocity[i].is_finite() {
                values[i]=target[i];
                self.velocity[i]=0.0;
            }
            if (values[i]-target[i]).abs()<0.0005 && self.velocity[i].abs()<0.0005 {
                values[i]=target[i];
                self.velocity[i]=0.0;
            }
        }
        self.current=Pose::from(values);
        // Render-only bounds, not movement or game-state constraints.
        self.current.yaw=self.current.yaw.clamp(-2.0,2.0);
        self.current.pitch=self.current.pitch.clamp(-3.0,3.0);
        self.current.roll=self.current.roll.clamp(-1.25,1.25);
        self.current.fov=self.current.fov.clamp(-4.0,4.0);
        self.current
    }
    /// Controls remain unaffected. Camera angles are additive over the
    /// already-computed FirstPerson/CameraOverhaul output from the engine.
    pub fn apply(&self, yaw:f32,pitch:f32,roll:f32,fov:f32) -> (f32,f32,f32,f32) {
        if !self.config.enabled || self.config.reduced_motion {
            // Preserve vanilla camera values and custom FoV configurations.
            return (yaw,pitch,roll,fov);
        }
        let accent=self.interaction_age.map_or(0.0, |age|
            (-7.0*age).exp()*(20.0*age).sin()
        );
        (
            yaw+self.current.yaw+accent*0.12,
            (pitch+self.current.pitch+accent*0.3).clamp(-89.9,89.9),
            roll+self.current.roll+accent*0.18,
            fov+self.current.fov,
        )
    }
}
/// Some native furnace and brewing screens are block-entity containers,
/// not entries in the station logic enum.
pub fn station_of_container_block(name: &str) -> Option<Station> {
    Some(match name {
        "minecraft:furnace" | "minecraft:blast_furnace" | "minecraft:smoker" => Station::Furnace,
        "minecraft:brewing_stand" => Station::Brewing,
        _ => return None,
    })
}

impl Game<'_> {
    /// Identify workstation scenes from the same state that renders the
    /// inventory screen. No server packets, block edits, or click interception.
    pub(super) fn cinematic_station(&self) -> Option<Station> {
        if !self.camera_type.first_person() || self.title.is_some()
            || self.screen != Screen::Inventory { return None; }
        if self.craft_table.is_some() { return Some(Station::Crafting); }
        if let Some(pos) = self.container {
            let block=self.world.block(pos.0,pos.1,pos.2);
            let name = &self.res.facts().get(block).name;
            if let Some(station) = station_of_container_block(name) {
                return Some(station);
            }
        }
        let kind=self.station.as_ref()?.kind;
        Some(match kind {
            stations::Kind::Enchanting => Station::Enchanting,
            stations::Kind::Anvil => Station::Anvil,
            stations::Kind::Grindstone => Station::Grindstone,
            stations::Kind::Smithing => Station::Smithing,
            stations::Kind::Stonecutter => Station::Stonecutter,
            stations::Kind::Merchant => Station::Other,
            stations::Kind::Beacon => Station::Beacon,
            stations::Kind::Cartography => Station::Cartography,
            stations::Kind::Loom => Station::Loom,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn scene_open_close_springs_to_rest() {
        let mut r=Rig::new(Config{enabled:true,..Default::default()});
        r.advance(Some(Station::Crafting),0.0);
        for i in 1..90 { r.advance(Some(Station::Crafting),i as f32/60.0); }
        assert!((r.current.pitch-Station::Crafting.target().pitch).abs()<0.05);
        for i in 90..180 { r.advance(None,i as f32/60.0); }
        assert!(r.current.pitch.abs()<0.03);
    }
    #[test] fn frame_step_limit_prevents_jump_after_resume() {
        let mut r=Rig::new(Config{enabled:true,..Default::default()});
        r.advance(Some(Station::Anvil),0.0);
        let p=r.advance(Some(Station::Anvil),900.0);
        assert!(p.pitch>0.0 && p.pitch<1.8);
    }
    #[test] fn maps_real_furnace_and_brewing_container_blocks() {
        for kind in ["minecraft:furnace","minecraft:smoker","minecraft:blast_furnace"] {
            assert_eq!(station_of_container_block(kind), Some(Station::Furnace));
        }
        assert_eq!(station_of_container_block("minecraft:brewing_stand"),Some(Station::Brewing));
        assert_eq!(station_of_container_block("minecraft:chest"),None);
    }
    #[test] fn scene_profiles_differ_and_stay_small() {
        assert_ne!(Station::Anvil.target(), Station::Enchanting.target());
        for s in [Station::Crafting,Station::Furnace,Station::Brewing,
            Station::Enchanting,Station::Anvil,Station::Grindstone,Station::Smithing,
            Station::Stonecutter,Station::Cartography,Station::Loom,Station::Beacon] {
            let p=s.target();
            assert!(p.roll.abs()<1.0 && p.pitch.abs()<3.0);
        }
    }
    #[test] fn disabling_reduced_motion_resets_pose() {
        let mut r=Rig::new(Config{enabled:true,..Default::default()});
        r.advance(Some(Station::Smithing),0.0);
        r.advance(Some(Station::Smithing),0.1);
        r.config.reduced_motion=true;
        assert_eq!(r.advance(Some(Station::Smithing),0.2),Pose::default());
    }
    #[test] fn workstation_actions_add_small_transient_camera_motion() {
        let mut r=Rig::new(Config{enabled:true,..Default::default()});
        r.advance(Some(Station::Anvil),0.0);
        r.advance(Some(Station::Anvil),0.1);
        let before=r.apply(0.0,0.0,0.0,75.0);
        r.pulse();
        r.advance(Some(Station::Anvil),0.14);
        let after=r.apply(0.0,0.0,0.0,75.0);
        assert_ne!(before.2,after.2);
        for i in 15..180 { r.advance(None,i as f32/60.0); }
        assert_eq!(r.interaction_age,None);
        assert!(r.current.roll.abs()<0.01);
    }
    #[test] fn disabled_interaction_never_pulses() {
        let mut r=Rig::default();
        r.pulse();
        assert_eq!(r.interaction_age,None);
    }
    #[test] fn clamp_applies_after_existing_camera() {
        let mut r=Rig::new(Config{enabled:true,..Default::default()});
        r.advance(Some(Station::Enchanting),0.0);
        r.advance(Some(Station::Enchanting),0.3);
        let (_y,p,_r,f)=r.apply(15.0,89.0,0.0,31.0);
        assert!(p<=89.9 && p>=-89.9 && f<31.0);
    }
}
