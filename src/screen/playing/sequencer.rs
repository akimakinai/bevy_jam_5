#![allow(unused)]
#![warn(unused_imports, dead_code)]

// This module is responsible for the sequencer UI.

// The sequencer UI has the seek bar. Receives play time via Resource?
// When the seek bar is dragged, emits rollback/rollforward event.
// Supplies what not is being played via Resource? Event (to be observed)?

// Btw, events and observers shares the same `Event` trait is communication hazard!
// "Send a Event" might mean "Send a Event to the Event system" or "Send a Event to the Observer".

use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};

use bevy_debug_text_overlay::{screen_print, OverlayPlugin};
use sickle_ui_scaffold::drag_interaction::{
    DragInteractionPlugin, DragState, Draggable, DraggableUpdate,
};
use sickle_ui_scaffold::drop_interaction::{DropInteractionPlugin, DropPhase, DropZone, Droppable};
use sickle_ui_scaffold::flux_interaction::{FluxInteractionPlugin, TrackedInteraction};

use crate::screen::Screen;
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        FluxInteractionPlugin,
        DragInteractionPlugin,
        DropInteractionPlugin,
    ));

    #[cfg(feature = "dev")]
    app.add_plugins(OverlayPlugin::default());

    app.add_systems(OnEnter(Screen::Playing), enter_playing);

    app.add_systems(
        Update,
        (note_drag, note_move_between_tracks, note_move_inactive)
            .chain()
            .run_if(in_state(Screen::Playing))
            .after(DraggableUpdate),
    );
}

#[derive(Component)]
struct Track;

fn enter_playing(mut commands: Commands) {
    commands
        .ui_root_with_style(|style| Style {
            justify_content: JustifyContent::End,
            ..style
        })
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.label("Sequencer");

            children
                .spawn(NodeBundle {
                    style: Style {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::auto(2),
                        row_gap: Val::Px(0.0),
                        column_gap: Val::Px(12.0),
                        align_content: AlignContent::Center,
                        ..Default::default()
                    },
                    ..default()
                })
                .with_children(|children| {
                    for s in ["Jump", "Attack", "Wait", "Backward"] {
                        children.spawn(TextBundle::from_section(
                            s,
                            TextStyle {
                                font_size: 24.0,
                                color: WHITE.into(),
                                ..default()
                            },
                        ));

                        children
                            .spawn((
                                Name::new(format!("Track {}", s)),
                                Track,
                                NodeBundle {
                                    style: Style {
                                        width: Px(500.0),
                                        height: Px(50.0),
                                        border: UiRect::all(Px(3.0)),
                                        overflow: Overflow::clip(),
                                        ..default()
                                    },
                                    border_color: BLUE_200.into(),
                                    ..default()
                                },
                                Interaction::None,
                                DropZone::default(),
                                RelativeCursorPosition::default(),
                            ))
                            .with_children(|children| {
                                children
                                    .spawn(
                                        TextBundle::from_section(
                                            "Drag me",
                                            TextStyle {
                                                font_size: 24.0,
                                                color: BLACK.into(),
                                                ..default()
                                            },
                                        )
                                        .with_style(
                                            Style {
                                                width: Px(100.0),
                                                height: Percent(100.0),
                                                position_type: PositionType::Absolute,
                                                align_content: AlignContent::Center,
                                                ..default()
                                            },
                                        ),
                                    )
                                    .insert((
                                        BackgroundColor(WHITE.into()),
                                        Interaction::None,
                                        TrackedInteraction::default(),
                                        Draggable::default(),
                                        Droppable::default(),
                                        RelativeCursorPosition::default(),
                                        Note,
                                    ));
                            });
                    }
                });
        });
}

#[derive(Component)]
struct Note;

#[derive(Component)]
struct OriginalTrack(Entity);

/// Update the position of a dragged note.
fn note_drag(
    mut commands: Commands,
    mut cards: Query<(Entity, &Draggable, &Parent, &mut Style), (With<Note>, Changed<Draggable>)>,
) {
    for (entity, draggable, parent, mut style) in cards.iter_mut() {
        if draggable.state == DragState::DragStart {
            commands.entity(entity).insert(OriginalTrack(parent.get()));
        }

        if matches!(draggable.state, DragState::DragStart | DragState::Dragging) {
            if let (Some(origin), Some(position)) = (draggable.origin, draggable.position) {
                let diff = position - origin;
                style.left = Px(diff.x);
            }
        }
    }
}

// Moves a note to other track during DroppableEntered|DroppableHover
fn note_move_between_tracks(
    mut commands: Commands,
    drop_zones: Query<(Entity, &DropZone), (Changed<DropZone>, With<Track>)>,
    notes: Query<(&Note, &Parent)>,
    name: Query<&Name>,
) {
    let mut track_infos = String::new();
    for (track_id, drop_zone) in drop_zones.iter() {
        if matches!(
            drop_zone.drop_phase(),
            DropPhase::DroppableEntered | DropPhase::DroppableHover
        ) {
            let Some(incoming) = drop_zone.incoming_droppable() else {
                continue;
            };
            let Ok((note, old_track)) = notes.get(incoming) else {
                continue;
            };

            // Move the note to the new track
            if old_track.get() != track_id {
                commands.entity(incoming).set_parent(track_id);
                screen_print!(
                    "Moved note {incoming:?} from track {:?} to {:?}",
                    name.get(old_track.get()).unwrap(),
                    name.get(track_id).unwrap()
                );

                // TODO: play sfx when note moved between tracks
                // TODO: change overlapped note's length?
            }
        }
    }
}

// If no `DropZone` is active while a note is being dragged, move it back to its original track.
// (`DropZone` of the original track is not activated by dragging `Draggable` from the same track)
fn note_move_inactive(
    mut commands: Commands,
    notes: Query<(Entity, &Draggable, &Parent, &OriginalTrack), With<Note>>,
    drop_zones: Query<&DropZone, With<Track>>,
) {
    let all_inactive = drop_zones
        .iter()
        .all(|drop_zone| drop_zone.drop_phase() == DropPhase::Inactive);
    if all_inactive {
        for (note_id, draggable, parent, orig_track) in notes.iter() {
            if matches!(draggable.state, DragState::DragStart | DragState::Dragging) {
                if parent.get() != orig_track.0 {
                    commands.entity(note_id).set_parent(orig_track.0);
                }
            }
        }
    }
}