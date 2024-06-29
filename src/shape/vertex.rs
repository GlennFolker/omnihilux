use std::{hash::Hash, marker::PhantomData};

use bevy::{
    core::Pod,
    ecs::system::{StaticSystemParam, SystemParam, SystemParamItem},
    prelude::*,
    render::{
        render_resource::{RenderPipelineDescriptor, VertexAttribute},
        Render, RenderApp,
    },
};

use crate::shape::{pipeline::Requests, ShapeSystems};

pub struct ShaperPlugin<T: Shaper> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: Shaper> Default for ShaperPlugin<T> {
    #[inline]
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T: Shaper> Plugin for ShaperPlugin<T> {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(ExtractSchedule, T::extract.in_set(ShapeSystems::ExtractShaper))
                .add_systems(Render, queue_drawers::<T>.in_set(ShapeSystems::QueueShaper));
        }
    }
}

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

pub trait Shaper: Component {
    type ExtractParam: SystemParam + 'static;
    type DrawParam: SystemParam + 'static;
    type Vertex: Vertex;

    fn extract(param: StaticSystemParam<Self::ExtractParam>);

    fn draw(&mut self, param: &mut SystemParamItem<Self::DrawParam>, out: &mut Vec<Request<Self::Vertex>>);
}

pub struct Request<T: Vertex> {
    pub layer: f32,
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,
    pub key: T::Key,
}

pub fn queue_drawers<T: Shaper>(
    mut query: Query<&mut T>,
    param: StaticSystemParam<T::DrawParam>,
    requests: Res<Requests<T::Vertex>>,
    mut vertices: Local<Vec<Request<T::Vertex>>>,
) {
    let mut param = param.into_inner();
    for mut drawer in &mut query {
        drawer.draw(&mut param, &mut vertices);
    }

    requests.values.lock().unwrap().append(&mut vertices);
}
