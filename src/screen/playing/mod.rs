//! The screen state for the main game loop.

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::spawn::level::SpawnLevel;

pub mod sequencer;

pub(super) fn plugin(app: &mut App) {
    // Sub-state of Screen::Playing
    app.add_sub_state::<SequencerState>();

    app.add_plugins(sequencer::plugin);

    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_playing(
    mut commands: Commands,
    mut proj: Query<&mut OrthographicProjection>,
    mut seq_state: ResMut<NextState<SequencerState>>,
) {
    commands.trigger(SpawnLevel);

    let mut proj = proj.single_mut();
    proj.scale = 0.5;
}

fn exit_playing(mut proj: Query<&mut OrthographicProjection>) {
    let mut proj = proj.single_mut();
    proj.scale = 1.0;
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

// This game was about playing a sequencer multiple times to solve puzzles.
// Now I don't think this is a good idea. A player needs to wait too long while just watching the sequencer play.
// Instead, let's make the all replayed sequencers play at the same time.
// No more "cycles", but OK. Not in time with the jam anyway.

// During seeking, use VideoGlitchSettings to simulate a VHS tape being rewound or fast-forwarded.
// Maybe add tween to intensity of the glitch.

#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[source(Screen = Screen::Playing)]
pub enum SequencerState {
    /// Playing the sequencer
    // If the user added a note during play, go to Seeking state and unwind play time to where the note is added
    Playing,
    /// Seeking animation is playing
    // does not respond to UI interaction?
    // Seeking to where?
    Seeking,
    /// Sequencer stopped initially. Transitions to `Playing` state once the user adds the first note.
    #[default]
    Stopped,
}
