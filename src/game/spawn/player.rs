//! Spawn the player.

use bevy::prelude::*;
use bevy_ecs_ldtk::{app::LdtkEntityAppExt, LdtkEntity};

pub(super) fn plugin(app: &mut App) {
    // app.observe(spawn_player);

    app.register_ldtk_entity::<Player>("Player");

    app.add_systems(Update, spawn_player);
}

// #[derive(Event, Debug)]
// pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Default, LdtkEntity)]
pub struct Player {}

fn spawn_player(mut commands: Commands, player_query: Query<(Entity, &Player, &Transform)>) {
    for (entity, _player, transform) in &player_query {
        commands.entity(entity).insert(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 1.0, 0.0),
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            transform: *transform,
            ..Default::default()
        });
    }
}

// fn spawn_player(
//     _trigger: Trigger<SpawnPlayer>,
//     mut commands: Commands,
//     image_handles: Res<HandleMap<ImageKey>>,
//     mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
// ) {
//     // A texture atlas is a way to split one image with a grid into multiple sprites.
//     // By attaching it to a [`SpriteBundle`] and providing an index, we can specify which section of the image we want to see.
//     // We will use this to animate our player character. You can learn more about texture atlases in this example:
//     // https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
//     let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
//     let texture_atlas_layout = texture_atlas_layouts.add(layout);
//     let player_animation = PlayerAnimation::new();

//     commands.spawn((
//         Name::new("Player"),
//         Player {},
//         SpriteBundle {
//             texture: image_handles[&ImageKey::Ducky].clone_weak(),
//             transform: Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
//             ..Default::default()
//         },
//         TextureAtlas {
//             layout: texture_atlas_layout.clone(),
//             index: player_animation.get_atlas_index(),
//         },
//         MovementController::default(),
//         Movement { speed: 420.0 },
//         WrapWithinWindow,
//         player_animation,
//         StateScoped(Screen::Playing),
//     ));
// }
