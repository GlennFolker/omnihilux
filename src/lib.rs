use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use iyes_progress::ProgressPlugin;

use crate::draw::{core::CVertex, DrawPlugin};

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
        ))
        .run()
}
