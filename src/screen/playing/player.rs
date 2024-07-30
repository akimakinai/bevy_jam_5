use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::{app::LdtkEntityAppExt, LdtkEntity};
use bevy_tnua::prelude::{
    TnuaBuiltinWalk, TnuaController, TnuaControllerBundle, TnuaControllerPlugin,
};
use bevy_tnua_avian2d::TnuaAvian2dPlugin;

use super::SequencerPlaying;

pub(super) fn plugin(app: &mut App) {
    // app.observe(spawn_player);

    app.add_plugins((
        PhysicsPlugins::default(),
        TnuaControllerPlugin::default(),
        TnuaAvian2dPlugin::default(),
    ));

    app.register_ldtk_entity::<Player>("Player");

    app.add_systems(Update, spawn_player);

    app.add_systems(
        Update,
        player_auto_movement.run_if(in_state(SequencerPlaying(true))),
    );
    app.add_systems(
        Update,
        player_auto_movement_stop.run_if(in_state(SequencerPlaying(false))),
    );
}

// #[derive(Event, Debug)]
// pub struct SpawnPlayer;

#[derive(Component, Debug, Clone, Default, LdtkEntity)]
pub struct Player {}

fn spawn_player(
    mut commands: Commands,
    player_query: Query<(Entity, &Player, &Transform), Added<Player>>,
) {
    for (entity, _player, transform) in &player_query {
        commands
            .entity(entity)
            .insert((
                RigidBody::Dynamic,
                Collider::capsule(8.0, 16.0),
                TnuaControllerBundle::default(),
            ))
            .with_children(|children| {
                children.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0),
                        custom_size: Some(Vec2::splat(16.0)),
                        ..default()
                    },
                    transform: *transform,
                    ..Default::default()
                });
            });
    }
}

fn player_auto_movement(mut player_query: Query<&mut TnuaController, With<Player>>) {
    for mut controller in &mut player_query {
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: Vec3::new(10.0, 0.0, 0.0),
            float_height: 2.0,
            max_slope: std::f32::consts::FRAC_PI_4,
            ..default()
        });
    }
}

fn player_auto_movement_stop(mut player_query: Query<&mut TnuaController, With<Player>>) {
    for mut controller in &mut player_query {
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: Vec3::new(0.0, 0.0, 0.0),
            float_height: 2.0,
            max_slope: std::f32::consts::FRAC_PI_4,
            ..default()
        });
    }
}
