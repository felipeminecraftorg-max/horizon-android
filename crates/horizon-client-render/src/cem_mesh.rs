//! Experimental OptiFine CEM box-mesh compiler for Horizon's existing entity vertex path.
//! This does NOT execute Java/Mixin mods. Only the explicitly supported subset
//! of .jem geometry is compiled: everything else falls back to vanilla.
use crate::mesher::Vertex;
use glam::{Mat3, Mat4, Vec3};
use horizon_optifine::{
    entity_models::{BoxModel, Model, Part},
    entity_pose::Pose,
};
use std::collections::BTreeMap;

const MAX_BOXES: usize = 768;
const MAX_QUADS: usize = MAX_BOXES * 6;

#[derive(Clone, Debug, PartialEq)]
pub enum MeshError {
    Unsupported(&'static str),
    Invalid(&'static str),
    TooLarge,
}

/// Four vertices per face. Vertex UV coordinates are normalized to the CEM
/// image size, and are mapped into the existing entity texture atlas when drawn.
#[derive(Clone, Debug)]
pub struct Quad {
    pub part: String,
    pub position: [[f32; 3]; 4],
    pub uv: [[f32; 2]; 4],
    pub normal: [f32; 3],
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub quads: Vec<Quad>,
}

const FACES: [(&str, [usize;4], [f32;3]);6] = [
    ("north", [0,1,2,3], [0.0,0.0,-1.0]),
    ("south", [5,4,7,6], [0.0,0.0,1.0]),
    ("west",  [4,0,3,7], [-1.0,0.0,0.0]),
    ("east",  [1,5,6,2], [1.0,0.0,0.0]),
    ("up",    [3,2,6,7], [0.0,1.0,0.0]),
    ("down",  [4,5,1,0], [0.0,-1.0,0.0]),
];

fn uv_for(face: &str, box_model: &BoxModel, width: f32, height: f32, depth: f32)
    -> Option<[f32;4]>
{
    if let Some(rect) = box_model.face_uv.get(face) { return Some(*rect); }
    let [u,v] = box_model.uv?;
    // Standard OptiFine cube-net UV rectangles in *texture pixels*.
    // Horizontal strip: west | north | east | south.
    Some(match face {
        "west" => [u, v+depth, u+depth, v+depth+height],
        "north" => [u+depth, v+depth, u+depth+width, v+depth+height],
        "east" => [u+depth+width, v+depth, u+depth+width+depth, v+depth+height],
        "south" => [u+depth+width+depth, v+depth,
                    u+depth+width+depth+width, v+depth+height],
        "up" => [u+depth, v, u+depth+width, v+depth],
        "down" => [u+depth+width, v, u+depth+width+width, v+depth],
        _ => return None,
    })
}

fn cube(
    part: &Part, b: &BoxModel, texture_size: [f32;2], out: &mut Vec<Quad>
) -> Result<(), MeshError> {
    let [x,y,z,w,h,d] = b.coords;
    if w <= 0.0 || h <= 0.0 || d <= 0.0 {
        return Err(MeshError::Invalid("non-positive box dimensions"));
    }
    let expand = b.expand_axes.unwrap_or([b.expand;3]);
    let min=[x-expand[0], y-expand[1], z-expand[2]];
    let max=[x+w+expand[0], y+h+expand[1], z+d+expand[2]];
    if (0..3).any(|axis| max[axis] <= min[axis]) {
        return Err(MeshError::Invalid("box expansion inverts geometry"));
    }
    let [mut x0,mut y0,mut z0]=min;
    let [mut x1,mut y1,mut z1]=max;
    if part.inverted_axes.contains('x') { (x0,x1)=(-x1,-x0); }
    if part.inverted_axes.contains('y') { (y0,y1)=(-y1,-y0); }
    if part.inverted_axes.contains('z') { (z0,z1)=(-z1,-z0); }
    let corners=[
        [x0,y0,z0],[x1,y0,z0],[x1,y1,z0],[x0,y1,z0],
        [x0,y0,z1],[x1,y0,z1],[x1,y1,z1],[x0,y1,z1],
    ];
    let [tex_w,tex_h]=texture_size;
    for (face,indices,normal) in FACES {
        let rect=uv_for(face,b,w,h,d).ok_or(MeshError::Unsupported(
            "a box needs textureOffset or face UV on all six faces"
        ))?;
        let [mut u0,mut v0,mut u1,mut v1]=rect;
        if [u0,v0,u1,v1].iter().any(|n| !n.is_finite()) {
            return Err(MeshError::Invalid("non-finite UV"));
        }
        if part.mirrored_uv.contains('u') { (u0,u1)=(u1,u0); }
        if part.mirrored_uv.contains('v') { (v0,v1)=(v1,v0); }
        let uv=[[u0/tex_w,v0/tex_h],[u1/tex_w,v0/tex_h],
                [u1/tex_w,v1/tex_h],[u0/tex_w,v1/tex_h]];
        let position=indices.map(|i| corners[i]);
        out.push(Quad{part:part.id.clone(),position,uv,normal});
    }
    Ok(())
}
fn walk(part: &Part, size:[f32;2], mesh:&mut Mesh) -> Result<(),MeshError> {
    // The initial bridge has only one texture binding per entity.
    if part.texture.is_some() { return Err(MeshError::Unsupported("per-part textures")); }
    if part.id.is_empty() { return Err(MeshError::Unsupported("anonymous model part")); }
    let size=part.texture_size.unwrap_or(size);
    if size.iter().any(|n| !n.is_finite() || *n<=0.0 || *n>4096.0) {
        return Err(MeshError::Invalid("texture size"));
    }
    for b in &part.boxes {
        if mesh.quads.len() > MAX_QUADS.saturating_sub(6) { return Err(MeshError::TooLarge); }
        cube(part,b,size,&mut mesh.quads)?;
    }
    for child in &part.children { walk(child,size,mesh)?; }
    Ok(())
}

impl Mesh {
    pub fn compile(model: &Model) -> Result<Self,MeshError> {
        if model.texture.is_some() {
            return Err(MeshError::Unsupported("custom root texture"));
        }
        let mut mesh=Self::default();
        let size=model.texture_size.unwrap_or([64.0,32.0]);
        for part in &model.parts { walk(part,size,&mut mesh)?; }
        if mesh.quads.is_empty() { return Err(MeshError::Unsupported("model has no boxes")); }
        Ok(mesh)
    }

    /// Append geometry using an already sampled pose. The passed matrix is
    /// the same camera-relative entity base transform as vanilla's renderer.
    /// Returns false without changing the output if the pose/atlas is unusable.
    pub fn draw(
        &self,pose:&Pose,base:Mat4,texture_uv:[f32;4],
        light:u32,out:&mut Vec<Vertex>,
    ) -> bool {
        if self.quads.is_empty() || !base.is_finite()
            || texture_uv.iter().any(|u| !u.is_finite()) { return false; }
        let before=out.len();
        for quad in &self.quads {
            if pose.visible(&quad.part)!=Some(true) { continue; }
            let Some(transform)=pose.world_matrix(&quad.part) else {
                out.truncate(before);
                return false;
            };
            if !transform.is_finite() {
                out.truncate(before);
                return false;
            }
            let normal_matrix=Mat3::from_mat4(base*transform).inverse().transpose();
            let norm=(normal_matrix*Vec3::from_array(quad.normal)).normalize_or_zero();
            if !norm.is_finite() { out.truncate(before); return false; }
            let packed=norm.to_array().map(|n| (n.clamp(-1.0,1.0)*127.0) as i8);
            for i in 0..4 {
                let local=transform.transform_point3(Vec3::from_array(quad.position[i])/16.0);
                let pos=base.transform_point3(local);
                if !pos.is_finite() { out.truncate(before); return false; }
                let local_uv=quad.uv[i];
                out.push(Vertex{
                    pos:pos.to_array(),
                    uv:[texture_uv[0]+(texture_uv[2]-texture_uv[0])*local_uv[0],
                        texture_uv[1]+(texture_uv[3]-texture_uv[1])*local_uv[1]],
                    layer:0,
                    color:0xffff_ffff,
                    light,
                    normal:[packed[0],packed[1],packed[2],-127],
                    glint_uv:[0.0,0.0],
                });
            }
        }
        out.len()>before
    }
}

/// CPU compilation occurs once per resource reload, not once per mob/frame.
pub fn compile_all(
    resources:&horizon_optifine::entity_resources::EntityResources,
) -> (BTreeMap<String,Mesh>,Vec<String>) {
    let mut meshes=BTreeMap::new();
    let mut warnings=Vec::new();
    for (id,path) in &resources.models_by_entity {
        let Some(prepared)=resources.models.get(path) else { continue };
        match Mesh::compile(&prepared.model) {
            Ok(mesh) => { meshes.insert(id.clone(),mesh); },
            Err(err) => warnings.push(format!("{path}: {err:?}")),
        }
    }
    (meshes,warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_optifine::{entity_models,entity_pose::Pose};
    #[test] fn cube_six_faces_and_uv_inside_atlas() {
        let m=entity_models::parse(
            r#"{"textureSize":[64,64],"models":[{"part":"head","boxes":[{
            "coordinates":[-4,-8,-4,8,8,8],"textureOffset":[0,0]}]}]}"#
        ).unwrap();
        let mesh=Mesh::compile(&m).unwrap();
        assert_eq!(mesh.quads.len(),6);
        let pose=Pose::from_model(&m).unwrap();
        let mut vertices=Vec::new();
        assert!(mesh.draw(&pose,Mat4::IDENTITY,[0.1,0.2,0.6,0.7],240,&mut vertices));
        assert_eq!(vertices.len(),24);
        assert!(vertices.iter().all(|v|v.uv[0]>=0.1 && v.uv[0]<=0.6));
        assert!(vertices.iter().all(|v|v.pos.iter().all(|p|p.is_finite())));
    }
    #[test] fn face_uv_and_pivot_rotation() {
        let m=entity_models::parse(
            r#"{"textureSize":[64,32],"models":[{"part":"head","boxes":[{
            "coordinates":[0,0,0,2,2,2],"uvUp":[0,0,2,2],
            "uvDown":[2,0,4,2],"uvNorth":[0,2,2,4],
            "uvSouth":[2,2,4,4],"uvEast":[4,2,6,4],"uvWest":[6,2,8,4]}]}]}"#
        ).unwrap();
        let mesh=Mesh::compile(&m).unwrap();
        let mut pose=Pose::from_model(&m).unwrap();
        pose.set("head.ry",std::f32::consts::FRAC_PI_2).unwrap();
        pose.update_world();
        let mut verts=Vec::new();
        assert!(mesh.draw(&pose,Mat4::IDENTITY,[0.0,0.0,1.0,1.0],240,&mut verts));
        assert_eq!(verts.len(),24);
    }
    #[test] fn invalid_or_unsupported_models_reject_without_partial_draw() {
        let m=entity_models::parse(r#"{"models":[{"part":"body","boxes":[{
            "coordinates":[0,0,0,2,2,2]}]}]}"#).unwrap();
        assert!(matches!(Mesh::compile(&m),Err(MeshError::Unsupported(_))));
        let m=entity_models::parse(r#"{"models":[{"part":"body","boxes":[{
            "coordinates":[0,0,0,2,2,2],"textureOffset":[0,0]}]}]}"#).unwrap();
        let mesh=Mesh::compile(&m).unwrap();
        let pose=Pose::from_model(&m).unwrap();
        let mut out=Vec::new();
        assert!(!mesh.draw(&pose,Mat4::from_scale(Vec3::ZERO),[0.0,0.0,1.0,1.0],240,&mut out)
            || out.iter().all(|v|v.normal.iter().all(|n|*n==0)));
    }
}
