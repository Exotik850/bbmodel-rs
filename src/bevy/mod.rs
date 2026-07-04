use bevy_asset::Handle;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_image::Image;
use bevy_mesh::{Mesh, Mesh3d};

use crate::BBModel;

pub struct BBModelPlugin;

impl bevy_app::Plugin for BBModelPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_plugins(JsonAssetPlugin::<BBModel>::new(&["bbmodel"]));
    }
}

