use std::{hash::Hash, marker::PhantomData, sync::Mutex};

use bevy::{
    core::Pod,
    ecs::{
        query::{QueryItem, ReadOnlyQueryData},
        system::{ReadOnlySystemParam, StaticSystemParam, SystemParamItem},
    },
    prelude::*,
    render::{
        render_resource::{RenderPipelineDescriptor, VertexAttribute},
        Render, RenderApp,
    },
};

use crate::draw::{pipeline::Requests, DrawSystems};

pub struct DrawerPlugin<T: Drawer, Sys: System<In = (), Out = ()>> {
    extract_drawer: Mutex<Option<Sys>>,
    _marker: PhantomData<fn() -> T>,
}

impl<T: Drawer, Sys: System<In = (), Out = ()>> DrawerPlugin<T, Sys> {
    #[inline]
    pub fn new<Marker>(extract_drawer: impl IntoSystem<(), (), Marker, System = Sys>) -> Self {
        Self {
            extract_drawer: Mutex::new(Some(IntoSystem::into_system(extract_drawer))),
            _marker: PhantomData,
        }
    }
}

impl<T: Drawer, Sys: System<In = (), Out = ()>> Plugin for DrawerPlugin<T, Sys> {
    fn build(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_systems(
                    ExtractSchedule,
                    self.extract_drawer
                        .lock()
                        .unwrap()
                        .take()
                        .unwrap()
                        .in_set(DrawSystems::ExtractDrawer),
                )
                .add_systems(Render, queue_drawers::<T>.in_set(DrawSystems::QueueDrawer));
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

pub fn queue_drawers<T: Drawer>(
    param: StaticSystemParam<T::Param>,
    mut query: Query<(&mut T, T::Entity)>,
    requests: Res<Requests<T::Vertex>>,
    mut vertices: Local<Vec<Request<T::Vertex>>>,
) {
    for (mut drawer, e) in &mut query {
        drawer.draw(&param, e, &mut vertices);
    }

    requests.values.lock().unwrap().append(&mut vertices);
}
