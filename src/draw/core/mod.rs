use bevy::{
    asset::Handle,
    core::{Pod, Zeroable},
    prelude::Shader,
    render::render_resource::{VertexAttribute, VertexFormat},
};
use bevy::render::render_resource::RenderPipelineDescriptor;

use crate::draw::vertex::{Vertex, VertexKey};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CVertex {
    pub position: [f32; 2],
}

impl Vertex for CVertex {
    type Key = CKey;

    const SHADER: Handle<Shader> = Handle::weak_from_u128(213808777024918471717406675324426180314);
    const SHADER_SOURCE: &'static str = "shaders/draw.wgsl";

    const LAYOUT: &'static [VertexAttribute] = &[VertexAttribute {
        format: VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    }];
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum CKey {
    Standard,
}

impl VertexKey for CKey {
    fn specialize(self, _: &mut RenderPipelineDescriptor) {}
}
