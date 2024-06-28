use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_2d::Transparent2d,
    prelude::*,
    render::{render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render, RenderApp, RenderSet},
};
use iyes_progress::prelude::*;

use crate::{
    draw::{
        pipeline::{
            prepare_vertices_batch, prepare_vertices_bind_group, queue_vertices, Batch, DrawBatchCommand, DrawPipeline,
            Requests,
        },
        vertex::{DrawLayer, Vertex},
    },
    GameState,
};

pub mod core;
pub mod pipeline;
pub mod vertex;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum DrawSystems {
    ExtractDrawer,
    QueueDrawer,
    QueueVertices,
    PrepareBatch,
    PrepareBindGroup,
}

pub struct DrawPlugin<T: Vertex> {
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> Default for DrawPlugin<T> {
    #[inline]
    fn default() -> Self {
        Self { _marker: PhantomData }
    }
}

impl<T: Vertex> Plugin for DrawPlugin<T> {
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
                .init_resource::<SpecializedRenderPipelines<DrawPipeline<T>>>()
                .init_resource::<Requests<T>>()
                .init_resource::<Batch<T>>()
                .init_resource::<DrawLayer<T>>()
                .add_render_command::<Transparent2d, DrawBatchCommand<T>>()
                .configure_sets(
                    Render,
                    (
                        (DrawSystems::QueueDrawer, DrawSystems::QueueVertices).in_set(RenderSet::Queue),
                        DrawSystems::QueueVertices.after_ignore_deferred(DrawSystems::QueueDrawer),
                        DrawSystems::PrepareBatch.in_set(RenderSet::Prepare),
                        DrawSystems::PrepareBindGroup.in_set(RenderSet::PrepareBindGroups),
                    ),
                )
                .add_systems(
                    Render,
                    (
                        queue_vertices::<T>.in_set(DrawSystems::QueueVertices),
                        prepare_vertices_batch::<T>.in_set(DrawSystems::PrepareBatch),
                        prepare_vertices_bind_group::<T>.in_set(DrawSystems::PrepareBindGroup),
                    ),
                );
        }
    }

    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<DrawPipeline<T>>();
        }
    }
}
