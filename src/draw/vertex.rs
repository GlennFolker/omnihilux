use std::{hash::Hash, marker::PhantomData};

use bevy::{
    ecs::{
        query::{QueryItem, ReadOnlyQueryData},
        system::{ReadOnlySystemParam, SystemParamItem},
    },
    prelude::*,
    render::render_resource::{RenderPipelineDescriptor, VertexAttribute},
};
use bytemuck::Pod;

#[derive(Resource)]
pub struct DrawLayer<T: Vertex> {
    pub layer: f32,
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> DrawLayer<T> {
    #[inline]
    pub const fn new(layer: f32) -> Self {
        Self {
            layer,
            _marker: PhantomData,
        }
    }
}

impl<T: Vertex> Copy for DrawLayer<T> {}
impl<T: Vertex> Clone for DrawLayer<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Vertex> Default for DrawLayer<T> {
    #[inline]
    fn default() -> Self {
        Self::new(50.0)
    }
}

pub trait Vertex: Send + Sync + Pod {
    type Key: VertexKey;

    const SHADER: Handle<Shader>;
    const SHADER_SOURCE: &'static str;

    const LAYOUT: &'static [VertexAttribute];
}

pub trait VertexKey: Send + Sync + Clone + Eq + PartialEq + Hash {
    fn specialize(self, desc: &mut RenderPipelineDescriptor);
}

pub trait Drawer: Component {
    type Param: ReadOnlySystemParam;
    type Entity: ReadOnlyQueryData;
    type Vertex: Vertex;

    fn draw(
        &mut self,
        param: &SystemParamItem<Self::Param>,
        entity: QueryItem<Self::Entity>,
        out: &mut Vec<Request<Self::Vertex>>,
    );
}

pub struct Request<T: Vertex> {
    pub layer: f32,
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub key: T::Key,
}
