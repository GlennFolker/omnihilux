use bevy::{
    ecs::{query::QueryItem, system::SystemParamItem},
    log::{Level, LogPlugin},
    prelude::*,
};
use iyes_progress::ProgressPlugin;

use crate::draw::{
    core::{CKey, CVertex},
    vertex::{Drawer, Request},
    DrawPlugin,
};
use crate::draw::vertex::DrawerPlugin;

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
        requests.push(Request {
            layer: 0.0,
            vertices: vec![
                CVertex {
                    position: [-100.0, -100.0],
                    color: [1.0, 0.0, 0.0, 1.0],
                },
                CVertex {
                    position: [100.0, -100.0],
                    color: [0.0, 1.0, 0.0, 1.0],
                },
                CVertex {
                    position: [100.0, 100.0],
                    color: [0.0, 0.0, 1.0, 1.0],
                },
                CVertex {
                    position: [-100.0, 100.0],
                    color: [1.0, 1.0, 1.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 2, 3, 0],
            key: CKey::Standard,
        });
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
