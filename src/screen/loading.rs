//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use super::Screen;
use crate::game::assets::{HandleMap, LdtkKey};
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter_loading);
    app.add_systems(
        Update,
        update_loading_text
            .run_if(in_state(Screen::Loading).and_then(on_timer(Duration::from_millis(200)))),
    );
    app.add_systems(
        Update,
        continue_to_title.run_if(in_state(Screen::Loading).and_then(all_assets_loaded)),
    );
}

#[derive(Component)]
struct LoadingText(usize);

fn enter_loading(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loading))
        .with_children(|children| {
            children.label("Loading   ").insert(LoadingText(0));
        });
}

fn update_loading_text(
    mut wrapper: Query<(&mut LoadingText, &Children)>,
    mut text: Query<&mut Text>,
) {
    for (mut loading_text, children) in wrapper.iter_mut() {
        loading_text.0 = (loading_text.0 + 1) % 4;
        text.get_mut(children[0]).unwrap().sections[0].value =
            format!("Loading{:3}", ".".repeat(loading_text.0));
    }
}

fn all_assets_loaded(
    asset_server: Res<AssetServer>,
    // image_handles: Res<HandleMap<ImageKey>>,
    // sfx_handles: Res<HandleMap<SfxKey>>,
    // soundtrack_handles: Res<HandleMap<SoundtrackKey>>,
    ldtk_handles: Res<HandleMap<LdtkKey>>,
) -> bool {
    // image_handles.all_loaded(&asset_server)
    //     && sfx_handles.all_loaded(&asset_server)
    //     && soundtrack_handles.all_loaded(&asset_server)
    ldtk_handles.all_loaded(&asset_server)
}

fn continue_to_title(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
