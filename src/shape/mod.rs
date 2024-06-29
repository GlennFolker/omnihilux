use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render, RenderApp, RenderSet},
};
use iyes_progress::prelude::*;

use crate::{
    shape::{
        pipeline::{
            prepare_vertices_batch, prepare_vertices_bind_group, queue_vertices, Batch, DrawShapes, Requests, ShapePipeline,
        },
        vertex::{DrawLayer, Vertex},
    },
    GameState,
};

pub mod pipeline;
pub mod vertex;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum ShapeSystems {
    ExtractShaper,
    QueueShaper,
    QueueVertices,
    PrepareBatch,
    PrepareBindGroup,
}

pub struct ShapePlugin<T: Vertex> {
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> Default for ShapePlugin<T> {
    #[inline]
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T: Vertex> Plugin for ShapePlugin<T> {
    fn build(&self, app: &mut App) {
        #[derive(Resource)]
        struct ShaderHandle(Handle<Shader>);

        app.add_systems(
            OnEnter(GameState::InitInternal),
            move |mut commands: Commands, server: Res<AssetServer>, mut loading: ResMut<AssetsLoading>| {
                let handle = server.load::<Shader>(T::SHADER_SOURCE);
                loading.add(&handle);

                commands.insert_resource(ShaderHandle(handle));
            },
        )
        .add_systems(
            OnExit(GameState::InitInternal),
            move |mut commands: Commands, handle: Res<ShaderHandle>, mut shaders: ResMut<Assets<Shader>>| {
                commands.remove_resource::<ShaderHandle>();

                let shader = shaders.remove(&handle.0).unwrap();
                shaders.insert(T::SHADER, shader);
            },
        );

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .init_resource::<SpecializedRenderPipelines<ShapePipeline<T>>>()
                .init_resource::<Requests<T>>()
                .init_resource::<Batch<T>>()
                .init_resource::<DrawLayer<T>>()
                .add_render_command::<Transparent2d, DrawShapes<T>>()
                .configure_sets(
                    Render,
                    (
                        (ShapeSystems::QueueShaper, ShapeSystems::QueueVertices).in_set(RenderSet::Queue),
                        ShapeSystems::QueueVertices.after_ignore_deferred(ShapeSystems::QueueShaper),
                        ShapeSystems::PrepareBatch.in_set(RenderSet::Prepare),
                        ShapeSystems::PrepareBindGroup.in_set(RenderSet::PrepareBindGroups),
                    ),
                )
                .add_systems(
                    Render,
                    (
                        queue_vertices::<T>.in_set(ShapeSystems::QueueVertices),
                        prepare_vertices_batch::<T>.in_set(ShapeSystems::PrepareBatch),
                        prepare_vertices_bind_group::<T>.in_set(ShapeSystems::PrepareBindGroup),
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<ShapePipeline<T>>();
        }
    }
}
