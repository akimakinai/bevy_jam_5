// #![allow(unused)]
// #![warn(unused_imports, dead_code)]

// This module is responsible for the sequencer UI.

// The sequencer UI has the seek bar. Receives play time via Resource.
// When the seek bar is dragged, emits rollback/rollforward event.
// Triggers event for what note is being played.

use bevy::color::palettes::css::WHITE;
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};

use bevy_debug_text_overlay::OverlayPlugin;
use sickle_ui_scaffold::drag_interaction::DragInteractionPlugin;
use sickle_ui_scaffold::drop_interaction::{DropInteractionPlugin, DropZone};
use sickle_ui_scaffold::flux_interaction::{FluxInteractionPlugin, TrackedInteraction};

use crate::screen::Screen;
use crate::ui::prelude::*;

mod notes;

pub use notes::{Note, NoteKind};

use super::SequencerPlaying;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        FluxInteractionPlugin,
        DragInteractionPlugin,
        DropInteractionPlugin,
    ));

    #[cfg(feature = "dev")]
    if !app.is_plugin_added::<OverlayPlugin>() {
        app.add_plugins(OverlayPlugin::default());
    }
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(Update, update_seek_bar.run_if(in_state(Screen::Playing)));

    app.add_systems(
        Update,
        (advance_play_pos, play_note)
            .run_if(in_state(Screen::Playing).and_then(in_state(SequencerPlaying(true)))),
    );

    app.add_plugins(notes::plugin);
}

#[derive(Resource)]
struct Sequencer {
    id: Entity,
    notes: Vec<Entity>,
    /// Play position in seconds.
    play_pos: f32,
}

#[derive(Component)]
struct Track;

const TRACK_WIDTH: f32 = 500.0;

#[derive(Component)]
struct SeekBar;

fn enter_playing(mut commands: Commands) {
    let mut seq_id = None;
    commands
        .ui_root_with_style(|style| Style {
            justify_content: JustifyContent::End,
            ..style
        })
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.label("Sequencer");

            seq_id = children
                .spawn((
                    Name::new("Sequencer"),
                    NodeBundle {
                        style: Style {
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::auto(2),
                            row_gap: Val::Px(0.0),
                            column_gap: Val::Px(12.0),
                            align_content: AlignContent::Center,
                            ..Default::default()
                        },
                        ..default()
                    },
                ))
                .with_children(|children| {
                    children.spawn((
                        Name::new("Seek Bar"),
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                height: Percent(100.0),
                                width: Px(5.0),
                                ..default()
                            },
                            z_index: ZIndex::Local(999),
                            background_color: BackgroundColor(BLUE_50.into()),
                            ..default()
                        },
                        SeekBar,
                    ));
                    // TODO: add seek bar handle

                    for s in ["Jump", "Attack", "Wait", "Backward"] {
                        children.spawn(TextBundle::from_section(
                            s,
                            TextStyle {
                                font_size: 24.0,
                                color: WHITE.into(),
                                ..default()
                            },
                        ));

                        children.spawn((
                            Name::new(format!("Track {}", s)),
                            Track,
                            NodeBundle {
                                style: Style {
                                    width: Px(TRACK_WIDTH),
                                    height: Px(50.0),
                                    border: UiRect::all(Px(3.0)),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                                border_color: BLUE_200.into(),
                                ..default()
                            },
                            Interaction::None,
                            TrackedInteraction::default(),
                            DropZone::default(),
                            RelativeCursorPosition::default(),
                        ));
                    }
                })
                .id()
                .into();
        });

    commands.insert_resource(Sequencer {
        id: seq_id.unwrap(),
        notes: vec![],
        play_pos: 0.0,
    });
}

fn exit_playing(mut commands: Commands) {
    commands.remove_resource::<Sequencer>();
}

// What the play position, in seconds, will be if we seek fully from the left to the right.
const SEQUENCER_WIDTH_TIME: f32 = 8.0;

fn get_clipped_rect(node: &Node, gt: &GlobalTransform, clip: Option<&CalculatedClip>) -> Rect {
    let rect = node.logical_rect(gt);
    clip.map(|clip| rect.intersect(clip.clip)).unwrap_or(rect)
}

// Reflects the current play position on the seek bar.
fn update_seek_bar(
    nodes: Query<(&Node, Option<&CalculatedClip>, &GlobalTransform)>,
    tracks: Query<Entity, With<Track>>,
    mut seek_bar: Query<&mut Style, With<SeekBar>>,
    sequencer: Res<Sequencer>,
) {
    // UI rect is not updated in first update, so we run this every frame
    // technically we can save rects first and only update when sequencer is changed
    // if !sequencer.is_changed() {
    //     return;
    // }

    // since the seek bar is a child of the sequencer and not a child of the track,
    // we need to calculate position based on sequencer's reference frame.

    let sequencer_rect = {
        let Ok((seq, seq_clip, seq_gt)) = nodes.get(sequencer.id) else {
            debug!("Sequencer node not found");
            return;
        };
        get_clipped_rect(seq, seq_gt, seq_clip)
    };

    let track_rect = {
        // a random track node to get the rect
        let Some(track_id) = tracks.iter().next() else {
            debug!("No tracks found");
            return;
        };
        let Ok((track, track_clip, track_gt)) = nodes.get(track_id) else {
            debug!("Track node not found");
            return;
        };
        get_clipped_rect(track, track_gt, track_clip)
    };

    let rel_track_x_min = (track_rect.min - sequencer_rect.min).x;

    let Ok(mut style) = seek_bar.get_single_mut() else {
        debug!("Seek bar not found");
        return;
    };
    style.left = Px(sequencer.play_pos / SEQUENCER_WIDTH_TIME * TRACK_WIDTH + rel_track_x_min);
}

fn advance_play_pos(
    time: Res<Time>,
    mut sequencer: ResMut<Sequencer>,
    playing_state: ResMut<NextState<SequencerPlaying>>,
) {
    let delta = time.delta_seconds();
    sequencer.play_pos += delta;

    if sequencer.play_pos > SEQUENCER_WIDTH_TIME {
        sequencer.play_pos = 0.0;
        // TODO: playing_state.set(SequencerPlaying(false));
    }
}

#[derive(Resource, Debug)]
pub struct PlayingNotes(pub Vec<Note>);

fn play_note(
    sequencer: Res<Sequencer>,
    notes: Query<&Note>,
    mut played_notes: ResMut<PlayingNotes>,
) {
    // We need to keep sending Jump event while playing, instead of a single event for each note.
    // Tnua handling duration of pressing jumps etc. seems nice and I want to use that.
    // It should be easy: just use a Resource.

    let play_pos = sequencer.play_pos;

    let mut playing = vec![];

    for &id in &sequencer.notes {
        if let Ok(note) = notes.get(id) {
            if play_pos > note.pos && play_pos < note.pos + note.width {
                // TODO: don't play newly added note under the seek bar midway
                playing.push(*note);
            }
        }
    }

    *played_notes = PlayingNotes(playing);
}
