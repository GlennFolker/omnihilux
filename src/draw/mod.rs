pub mod pipeline;
pub mod vertex;

use std::marker::PhantomData;

use bevy::{
    asset::AssetPath,
    prelude::*,
    render::{render_resource::SpecializedRenderPipelines, Render, RenderApp, RenderSet},
};
use iyes_progress::prelude::*;

use crate::{
    draw::{
        pipeline::{Batch, DrawPipeline, Requests},
        vertex::{DrawLayer, Vertex},
    },
    GameState,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum DrawSystems {
    QueueDrawer,
    QueueVertices,
}

pub struct DrawPlugin<T: Vertex> {
    pub asset: AssetPath<'static>,
    _marker: PhantomData<fn(T)>,
}

impl<T: Vertex> DrawPlugin<T> {
    #[inline]
    pub fn new(path: impl Into<AssetPath<'static>>) -> Self {
        Self {
            asset: path.into(),
            _marker: PhantomData,
        }
    }
}

impl<T: Vertex> Plugin for DrawPlugin<T> {
    fn build(&self, app: &mut App) {
        #[derive(Resource)]
        struct ShaderHandle(Handle<Shader>);

        let asset = self.asset.clone();
        app.add_systems(
            OnEnter(GameState::InitInternal),
            move |mut commands: Commands, server: Res<AssetServer>, mut loading: ResMut<AssetsLoading>| {
                let handle = server.load::<Shader>(asset.clone());
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
                .configure_sets(
                    Render,
                    (
                        (DrawSystems::QueueDrawer, DrawSystems::QueueVertices).in_set(RenderSet::Queue),
                        DrawSystems::QueueVertices.after_ignore_deferred(DrawSystems::QueueDrawer),
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
