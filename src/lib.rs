use bevy::{
    core_pipeline::bloom::BloomSettings,
    prelude::*,
};
use iyes_progress::ProgressPlugin;

use crate::{
    draw::vertex::DrawVertex,
    entity::{blob::Blob, EntityPlugin},
    shape::ShapePlugin,
};

pub mod draw;
pub mod entity;
pub mod shape;
pub mod util;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum GameState {
    InitInternal,
    Init,
    Menu,
}

#[inline]
pub fn entry() {
    App::new()
        .insert_state(GameState::InitInternal)
        .add_plugins((
            DefaultPlugins,
            ProgressPlugin::new(GameState::InitInternal)
                .continue_to(GameState::Init)
                .track_assets(),
            ShapePlugin::<DrawVertex>::default(),
            EntityPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(OnEnter(GameState::Init), init)
        .run()
}

pub fn init(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera { hdr: true, ..default() },
            ..Camera2dBundle::new_with_far(1000.0)
        },
        BloomSettings {
            intensity: 0.5,
            ..BloomSettings::NATURAL
        },
    ));

    commands.spawn((TransformBundle::default(), Blob {
        border_color: Color::hex("#edcb4fff").unwrap() * 3.0,
        eye_color: Color::hex("#e92f70ff").unwrap() * 4.5,
        cell_color: Color::hex("#bd14c1ff").unwrap() * 1.5,
    }));
}
