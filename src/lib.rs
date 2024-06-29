use bevy::{
    ecs::{query::QueryItem, system::SystemParamItem},
    log::{Level, LogPlugin},
    prelude::*,
    render::render_resource::{BlendComponent, BlendFactor, BlendOperation, BlendState},
};
use iyes_progress::ProgressPlugin;

use crate::draw::{
    core::{CKey, CVertex},
    vertex::{Drawer, DrawerPlugin, Request},
    DrawPlugin,
};

pub mod draw;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum GameState {
    InitInternal,
    Init,
    Menu,
}

#[inline]
pub fn entry() {
    let mut def = DefaultPlugins.build();
    #[cfg(debug_assertions)]
    {
        def = def.set(LogPlugin {
            level: if cfg!(debug_assertions) { Level::DEBUG } else { Level::INFO },
            ..default()
        });
    }

    App::new()
        .insert_state(GameState::InitInternal)
        .add_plugins((
            def,
            ProgressPlugin::new(GameState::InitInternal)
                .continue_to(GameState::Init)
                .track_assets(),
            DrawPlugin::<CVertex>::default(),
            DrawerPlugin::<CDraw, _>::new(extract),
        ))
        .add_systems(OnEnter(GameState::Init), init)
        .run()
}

#[derive(Component, Copy, Clone)]
struct CDraw;
impl Drawer for CDraw {
    type Param = ();
    type Entity = ();
    type Vertex = CVertex;

    fn draw(
        &mut self,
        _: &SystemParamItem<Self::Param>,
        _: QueryItem<Self::Entity>,
        requests: &mut Vec<Request<Self::Vertex>>,
    ) {
        requests.extend([
            Request {
                layer: 0.0,
                vertices: vec![
                    CVertex {
                        position: [-150.0, -150.0],
                        color: [1.0, 0.0, 0.0, 0.5],
                    },
                    CVertex {
                        position: [50.0, -150.0],
                        color: [0.0, 1.0, 0.0, 0.5],
                    },
                    CVertex {
                        position: [50.0, 50.0],
                        color: [0.0, 0.0, 1.0, 0.5],
                    },
                    CVertex {
                        position: [-150.0, 50.0],
                        color: [1.0, 1.0, 1.0, 0.5],
                    },
                ],
                indices: vec![0, 1, 2, 2, 3, 0],
                key: default(),
            },
            Request {
                layer: 0.0,
                vertices: vec![
                    CVertex {
                        position: [-50.0, -50.0],
                        color: [1.0, 0.0, 0.0, 1.0],
                    },
                    CVertex {
                        position: [150.0, -50.0],
                        color: [0.0, 1.0, 0.0, 0.0],
                    },
                    CVertex {
                        position: [150.0, 150.0],
                        color: [0.0, 0.0, 1.0, 1.0],
                    },
                    CVertex {
                        position: [-50.0, 150.0],
                        color: [0.0, 0.0, 0.0, 0.0],
                    },
                ],
                indices: vec![0, 1, 2, 2, 3, 0],
                key: CKey {
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    }),
                    ..default()
                },
            },
        ]);
    }
}

fn extract(mut commands: Commands) {
    commands.spawn(CDraw);
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        camera: Camera { hdr: true, ..default() },
        ..Camera2dBundle::new_with_far(1000.0)
    });
}
