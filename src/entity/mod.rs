use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::{entity::blob::BlobShaper, shape::vertex::ShaperPlugin};

pub mod blob;

pub struct EntityPlugin;
impl PluginGroup for EntityPlugin {
    #[inline]
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(ShaperPlugin::<BlobShaper>::default())
    }
}
