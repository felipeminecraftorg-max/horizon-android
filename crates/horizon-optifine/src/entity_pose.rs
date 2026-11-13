//! Lightweight render-pose hierarchy for Custom Entity Models.
//!
//! CEM model parts are stored in model units; animation .rx/.ry/.rz values are
//! radians. This layer computes matrices only: it never touches entity physics,
//! camera, network state, textures, or GPU buffers.
use crate::entity_models::{Model, Part};
use glam::{EulerRot, Mat4, Vec3};
use std::collections::{BTreeMap, BTreeSet};

const MAX_PARTS: usize = 512;

#[derive(Clone, Debug, PartialEq)]
pub enum PoseError {
    TooManyParts, DuplicateId(String), MissingPart(String),
    InvalidTarget(String), InvalidValue,
}
#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub parent: Option<usize>,
    pub pivot: Vec3,
    pub rotation: Vec3,
    pub scale: Vec3,
    pub visible: bool,
    pub world: Mat4,
}
#[derive(Clone, Debug, Default)]
pub struct Pose {
    pub nodes: Vec<Node>,
    by_name: BTreeMap<String,usize>,
}
fn finite(n: f32) -> bool { n.is_finite() && n.abs() <= 100_000.0 }
impl Pose {
    pub fn from_model(model: &Model) -> Result<Self, PoseError> {
        let mut pose = Self::default();
        for p in &model.parts { pose.push_part(p, None)?; }
        pose.update_world();
        Ok(pose)
    }
    fn push_part(&mut self, part: &Part, parent: Option<usize>) -> Result<(), PoseError> {
        if self.nodes.len() == MAX_PARTS { return Err(PoseError::TooManyParts); }
        let id = if part.id.is_empty() { &part.name } else { &part.id };
        if id.is_empty() || self.by_name.contains_key(id) {
            return Err(PoseError::DuplicateId(id.into()));
        }
        if part.translation.iter().any(|x| !finite(*x))
            || part.rotation.iter().any(|x| !finite(*x)) { return Err(PoseError::InvalidValue); }
        let i = self.nodes.len();
        self.by_name.insert(id.into(), i);
        self.nodes.push(Node {
            id: id.into(), parent, pivot: Vec3::from_array(part.translation),
            rotation: Vec3::from_array(part.rotation.map(f32::to_radians)),
            scale: Vec3::ONE, visible: true, world: Mat4::IDENTITY,
        });
        for child in &part.children { self.push_part(child, Some(i))?; }
        Ok(())
    }
    /// Apply a compiled CEM expression to a specific local part channel.
    /// Does not mutate state on unknown channels or NaN/Infinity values.
    pub fn set(&mut self, target: &str, value: f32) -> Result<(), PoseError> {
        if !finite(value) { return Err(PoseError::InvalidValue); }
        let (id, channel) = target.rsplit_once('.')
            .ok_or_else(|| PoseError::InvalidTarget(target.into()))?;
        let Some(index) = self.by_name.get(id).copied() else {
            return Err(PoseError::MissingPart(id.into()));
        };
        let node = &mut self.nodes[index];
        match channel {
            "rx" => node.rotation.x = value,
            "ry" => node.rotation.y = value,
            "rz" => node.rotation.z = value,
            "tx" => node.pivot.x = value,
            "ty" => node.pivot.y = value,
            "tz" => node.pivot.z = value,
            "sx" => node.scale.x = value,
            "sy" => node.scale.y = value,
            "sz" => node.scale.z = value,
            "visible" => node.visible = value > 0.0,
            _ => return Err(PoseError::InvalidTarget(target.into())),
        }
        Ok(())
    }
    /// Recalculate parent-to-child local transforms after applying channels.
    pub fn update_world(&mut self) {
        for i in 0..self.nodes.len() {
            let n = &self.nodes[i];
            let local = Mat4::from_translation(n.pivot)
                * Mat4::from_euler(EulerRot::XYZ,n.rotation.x,n.rotation.y,n.rotation.z)
                * Mat4::from_scale(n.scale);
            self.nodes[i].world = match n.parent {
                Some(parent) => self.nodes[parent].world * local,
                None => local,
            };
        }
    }
    pub fn world_matrix(&self,id: &str) -> Option<Mat4> {
        self.by_name.get(id).map(|i| self.nodes[*i].world)
    }
    pub fn visible(&self,id: &str) -> Option<bool> {
        let mut index = *self.by_name.get(id)?;
        let mut ancestors = BTreeSet::new();
        loop {
            if !ancestors.insert(index) || !self.nodes[index].visible { return Some(false); }
            match self.nodes[index].parent {
                Some(parent) => index = parent,
                None => return Some(true),
            }
        }
    }
    /// Sample precompiled model expressions and apply supported channels.
    /// A failed channel is reported, not applied, so callers can fall back.
    pub fn apply_sample(
        &mut self, samples: &[(String,f32)],
    ) -> Vec<(String, PoseError)> {
        let mut errors = Vec::new();
        for (target,value) in samples {
            if let Err(err) = self.set(target,*value) { errors.push((target.clone(),err)); }
        }
        self.update_world();
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity_models;

    #[test]
    fn animated_head_rotation_is_in_radians() {
        let model=entity_models::parse(r#"{"models":[{"part":"head","rotate":[0,0,0]}]}"#).unwrap();
        let mut pose=Pose::from_model(&model).unwrap();
        assert!(pose.apply_sample(&[("head.rx".into(),std::f32::consts::FRAC_PI_2)]).is_empty());
        let transformed=pose.world_matrix("head").unwrap().transform_vector3(Vec3::Y);
        assert!((transformed.z-1.0).abs()<0.0001);
    }
    #[test]
    fn nested_part_follows_parent() {
        let model=entity_models::parse(r#"{"models":[{"part":"body","translate":[1,0,0],
            "submodels":[{"id":"arm","translate":[2,0,0]}]}]}"#).unwrap();
        let pose=Pose::from_model(&model).unwrap();
        assert_eq!(pose.world_matrix("arm").unwrap().transform_point3(Vec3::ZERO).x,3.0);
    }
    #[test]
    fn visibility_propagates_and_invalid_targets_fail() {
        let model=entity_models::parse(r#"{"models":[{"id":"root","submodels":[{"id":"child"}]}]}"#).unwrap();
        let mut pose=Pose::from_model(&model).unwrap();
        assert!(pose.apply_sample(&[("root.visible".into(),0.0)]).is_empty());
        assert_eq!(pose.visible("child"),Some(false));
        assert!(matches!(pose.set("bad.rx",1.0),Err(PoseError::MissingPart(_))));
        assert!(matches!(pose.set("root.rx",f32::NAN),Err(PoseError::InvalidValue)));
    }
}
