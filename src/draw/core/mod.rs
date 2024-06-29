use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    render::render_resource::{BufferAddress, RenderPipelineDescriptor, VertexAttribute, VertexFormat},
};

use crate::draw::vertex::{Vertex, VertexKey};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct CVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex for CVertex {
    type Key = CKey;

    const SHADER: Handle<Shader> = Handle::weak_from_u128(213808777024918471717406675324426180314);
    const SHADER_SOURCE: &'static str = "shaders/draw.wgsl";

    const LAYOUT: &'static [VertexAttribute] = &[
        VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        },
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: size_of::<[f32; 2]>() as BufferAddress,
            shader_location: 1,
        },
    ];
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum CKey {
    Standard,
}

impl VertexKey for CKey {
    fn specialize(self, _: &mut RenderPipelineDescriptor) {}
}
