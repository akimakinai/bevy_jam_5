//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::spawn::level::SpawnLevel;

pub mod sequencer;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<SequencerPlaying>();

    app.add_plugins(sequencer::plugin);

    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

// defined here because it's used by both the sequencer and player systems
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub struct SequencerPlaying(pub bool);

fn enter_playing(
    mut commands: Commands,
    mut proj: Query<&mut OrthographicProjection>,
    mut seq_state: ResMut<NextState<SequencerPlaying>>,
) {
    commands.trigger(SpawnLevel);

    let mut proj = proj.single_mut();
    proj.scale = 0.5;

    seq_state.set(SequencerPlaying(true));
}

fn exit_playing(mut proj: Query<&mut OrthographicProjection>) {
    let mut proj = proj.single_mut();
    proj.scale = 1.0;
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
