use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    render::render_resource::{
        BlendState, BufferAddress, ColorWrites, RenderPipelineDescriptor, VertexAttribute, VertexFormat,
    },
};

use crate::shape::vertex::{Vertex, VertexKey};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct DrawVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex for DrawVertex {
    type Key = DrawKey;

    const SHADER: Handle<Shader> = Handle::weak_from_u128(213808777024918471717406675324426180314);
    const SHADER_SOURCE: &'static str = "shaders/shape.wgsl";

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
pub struct DrawKey {
    pub mask: ColorWrites,
    pub blend: Option<BlendState>,
}

impl Default for DrawKey {
    #[inline]
    fn default() -> Self {
        Self {
            mask: ColorWrites::ALL,
            blend: Some(BlendState::ALPHA_BLENDING),
        }
    }
}

impl VertexKey for DrawKey {
    fn specialize(self, desc: &mut RenderPipelineDescriptor) {
        for target in &mut desc.fragment.as_mut().unwrap().targets {
            if let Some(target) = target {
                target.write_mask = self.mask;
                target.blend = self.blend;
            }
        }
    }
}
