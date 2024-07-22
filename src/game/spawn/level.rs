//! Spawn the main level by triggering other observers.

use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkWorldBundle, LevelSelection};

use crate::{game::assets::{HandleMap, LdtkKey}, screen::Screen};

// use super::player::SpawnPlayer;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);

    app.insert_resource(LevelSelection::index(0));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    ldtk_handle: Res<HandleMap<LdtkKey>>,
) {
    // // The only thing we have in our level is a player,
    // // but add things like walls etc. here.
    // commands.trigger(SpawnPlayer);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: ldtk_handle[&LdtkKey::Level].clone(),
        ..Default::default()
    }).insert(StateScoped(Screen::Playing));
}
