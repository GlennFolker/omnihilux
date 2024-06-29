use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    render::render_resource::{
        BlendState, BufferAddress, ColorWrites, RenderPipelineDescriptor, VertexAttribute, VertexFormat,
    },
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
pub struct CKey {
    pub mask: ColorWrites,
    pub blend: Option<BlendState>,
}

impl Default for CKey {
    #[inline]
    fn default() -> Self {
        Self {
            mask: ColorWrites::ALL,
            blend: Some(BlendState::ALPHA_BLENDING),
        }
    }
}

impl VertexKey for CKey {
    fn specialize(self, desc: &mut RenderPipelineDescriptor) {
        for target in &mut desc.fragment.as_mut().unwrap().targets {
            if let Some(target) = target {
                target.write_mask = self.mask;
                target.blend = self.blend;
            }
        }
    }
}
