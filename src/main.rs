// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
#[cfg(feature = "dev_native")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_jam_5::AppPlugin;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins(AppPlugin);

    #[cfg(feature = "dev_native")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run()
}
