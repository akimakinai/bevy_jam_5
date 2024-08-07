use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_debug_text_overlay::screen_print;
use bevy_ecs_ldtk::prelude::*;
use bevy_tnua::{
    prelude::{
        TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController, TnuaControllerBundle,
        TnuaControllerPlugin,
    },
    TnuaUserControlsSystemSet,
};
use bevy_tnua_avian2d::{TnuaAvian2dPlugin, TnuaAvian2dSensorShape};

use super::{
    sequencer::{NoteKind, PlayingNotes},
    SequencerPlaying,
};

pub(super) fn plugin(app: &mut App) {
    // app.observe(spawn_player);

    app.add_plugins((
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        TnuaControllerPlugin::default(),
        TnuaAvian2dPlugin::default(),
    ))
    .insert_resource(Gravity(Vec2::NEG_Y * 100.0));

    // these probably should belong to spawn::level
    app.register_ldtk_entity::<Player>("Player")
        .observe(spawn_player);
    app.register_ldtk_int_cell::<WallBundle>(1)
        .observe(spawn_wall);

    app.add_systems(
        Update,
        run_played_note
            .run_if(in_state(SequencerPlaying(true)))
            .in_set(TnuaUserControlsSystemSet),
    );

    // Tnua control must be in the Update schedule
    app.add_systems(
        Update,
        (
            player_auto_movement.run_if(
                state_changed::<SequencerPlaying>.and_then(in_state(SequencerPlaying(true))),
            ),
            player_auto_movement_stop.run_if(
                state_changed::<SequencerPlaying>.and_then(in_state(SequencerPlaying(false))),
            ),
        )
            .in_set(TnuaUserControlsSystemSet),
    );
}

#[derive(Component, Debug, Clone, Default, LdtkEntity)]
pub struct Player {}

fn spawn_player(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    let entity = trigger.entity();

    // note: at this point, bevy_ecs_ldtk have not added Transform yet

    let collider = Collider::round_rectangle(13.0, 13.0, 1.0);

    commands
        .entity(entity)
        .insert((
            RigidBody::Dynamic,
            collider.clone(),
            TnuaControllerBundle::default(),
            TnuaAvian2dSensorShape(collider),
        ))
        .with_children(|children| {
            children.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0),
                    custom_size: Some(Vec2::splat(16.0)),
                    ..default()
                },
                ..Default::default()
            });
        });
}

fn player_auto_movement(mut player_query: Query<&mut TnuaController, With<Player>>) {
    for mut controller in &mut player_query {
        controller.basis(TnuaBuiltinWalk {
            // desired_velocity: Vec3::new(20.0, 0.0, 0.0),
            float_height: 4.0,
            max_slope: std::f32::consts::FRAC_PI_4,
            ..default()
        });
    }
}

fn player_auto_movement_stop(mut player_query: Query<&mut TnuaController, With<Player>>) {
    for mut controller in &mut player_query {
        controller.basis(TnuaBuiltinWalk {
            desired_velocity: Vec3::new(0.0, 0.0, 0.0),
            float_height: 4.0,
            max_slope: std::f32::consts::FRAC_PI_4,
            ..default()
        });
    }
}

#[derive(Default, Bundle, LdtkIntCell)]
struct WallBundle {
    wall: Wall,
}

#[derive(Default, Component)]
struct Wall;

/// Attaches Avian collider to a spawned wall entitiy.
fn spawn_wall(trigger: Trigger<OnAdd, Wall>, mut commands: Commands, coords: Query<&GridCoords>) {
    let entity = trigger.entity();

    // Does bevy_ecs_ldtk inserts GridCoords before or the same time as Wall?
    // Maybe we should listen for `LevelEvent`
    let Ok(coords) = coords.get(entity) else {
        return;
    };

    commands.entity(entity).insert((
        RigidBody::Static,
        Collider::rectangle(15.9, 15.9),
        Transform::from_translation(Vec3::new(coords.x as f32 * 16., coords.y as f32 * 16., 0.0)),
    ));
}

fn run_played_note(
    playing_notes: Res<PlayingNotes>,
    mut player: Query<&mut TnuaController, With<Player>>,
) {
    for note in &playing_notes.0 {
        debug!("Playing note: {:?}", note);
        let action = match note.kind {
            NoteKind::Jump => TnuaBuiltinJump {
                height: 32.0,
                ..default()
            },
        };

        // there may be a puzzle with multiple player characters...
        for mut controller in &mut player {
            controller.action(action.clone());
        }
    }
}
