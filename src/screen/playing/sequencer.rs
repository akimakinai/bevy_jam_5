// #![allow(unused)]
// #![warn(unused_imports, dead_code)]

// This module is responsible for the sequencer UI.

// The sequencer UI has the seek bar. Receives play time via Resource?
// When the seek bar is dragged, emits rollback/rollforward event.
// Supplies what not is being played via Resource? Event (to be observed)?

// Btw, events and observers shares the same `Event` trait is communication hazard!
// "Send a Event" might mean "Send a Event to the Event system" or "Send a Event to the Observer".

use std::ops::Deref;

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
use sickle_ui_scaffold::prelude::{FluxInteraction, FluxInteractionUpdate};

use crate::screen::Screen;
use crate::ui::prelude::*;

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

    app.register_type::<NoteDragged>();
    // #[cfg(feature = "dev")]
    // app.add_plugins(ResourceInspectorPlugin::<NoteDragged>::default());
    app.add_systems(
        Update,
        (note_drag, note_move_between_tracks, note_move_inactive)
            .chain()
            .run_if(in_state(Screen::Playing))
            .after(DraggableUpdate),
    );

    app.add_systems(
        Update,
        track_interaction
            .after(FluxInteractionUpdate)
            .run_if(in_state(Screen::Playing)),
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
                                TrackedInteraction::default(),
                                DropZone::default(),
                                RelativeCursorPosition::default(),
                            ))
                            .with_children(|children| {
                                children.spawn(Note::bundle(0.0));
                            });
                    }
                });
        });
}

#[derive(Component)]
struct Note;

const NOTE_DEFAULT_WIDTH: f32 = 100.0;

impl Note {
    fn bundle(left_px: f32) -> impl Bundle {
        (
            TextBundle::from_section(
                "Drag me",
                TextStyle {
                    font_size: 24.0,
                    color: BLACK.into(),
                    ..default()
                },
            )
            .with_style(Style {
                left: Px(left_px),
                width: Px(NOTE_DEFAULT_WIDTH),
                height: Percent(100.0),
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                ..default()
            })
            .with_background_color(WHITE.into()),
            Interaction::None,
            TrackedInteraction::default(),
            Draggable::default(),
            Droppable::default(),
            RelativeCursorPosition::default(),
            Note,
        )
    }
}

// If a track is pressed and no notes are hovered, spawn a note at the cursor position.
fn track_interaction(
    mut commands: Commands,
    notes: Query<&Interaction, With<Note>>,
    tracks: Query<
        (
            Entity,
            &Name,
            &Node,
            &GlobalTransform,
            &FluxInteraction,
            &RelativeCursorPosition,
        ),
        (With<Track>, Changed<FluxInteraction>),
    >,
) {
    let any_notes_interacted = notes
        .iter()
        .any(|&interaction| interaction != Interaction::None);
    if any_notes_interacted {
        return;
    }

    for (track_id, name, node, gtr, interaction, rel_cur_pos) in tracks.iter() {
        if interaction.is_pressed() {
            screen_print!("Track {:?} pressed", name);

            let cur_x = if let Some(touch_pos) = rel_cur_pos.normalized {
                touch_pos.x
            } else {
                // this could be unreachable!(), but let's be safe
                return;
            };

            commands.entity(track_id).with_children(|child| {
                let track_width = node.logical_rect(gtr).width();
                child.spawn(Note::bundle(track_width * cur_x - NOTE_DEFAULT_WIDTH / 2.0));
            });
        }
    }
}

#[derive(Resource, Clone, Debug, Reflect)]
struct NoteDragged {
    note: Entity,
    orig_track: Entity,
    orig_left: f32,
}

/// Update the position of a dragged note.
fn note_drag(
    mut commands: Commands,
    mut cards: Query<(Entity, &Draggable, &Parent, &mut Style), (With<Note>, Changed<Draggable>)>,
    note_drag: Option<ResMut<NoteDragged>>,
) {
    for (entity, draggable, parent, mut style) in cards.iter_mut() {
        if matches!(
            draggable.state,
            DragState::DragEnd | DragState::DragCanceled | DragState::Inactive
        ) {
            if let Some(note_drag) = &note_drag {
                if note_drag.note == entity {
                    commands.remove_resource::<NoteDragged>();
                }
            }
            continue;
        }

        if matches!(draggable.state, DragState::DragStart | DragState::Dragging) {
            let note_drag = if let Some(note_drag) = &note_drag {
                if note_drag.note != entity {
                    None
                } else {
                    Some(note_drag.deref().clone())
                }
            } else {
                None
            };

            let note_drag = note_drag.unwrap_or_else(|| {
                let px = if let Px(px) = style.left { px } else { 0.0 };
                let nd = NoteDragged {
                    note: entity,
                    orig_track: parent.get(),
                    orig_left: px,
                };
                commands.insert_resource(nd.clone());
                nd
            });

            if let (Some(origin), Some(position)) = (draggable.origin, draggable.position) {
                let diff = position - origin + note_drag.orig_left;
                style.left = Px(diff.x);
            }

            break;
        }
    }
}

// Moves a note to other track during DroppableEntered|DroppableHover
fn note_move_between_tracks(
    mut commands: Commands,
    drop_zones: Query<(Entity, &DropZone), (Changed<DropZone>, With<Track>)>,
    notes: Query<(&Note, &Parent)>,
    name: Query<&Name>,
    note_drag: Option<Res<NoteDragged>>,
) {
    let Some(note_drag) = note_drag else {
        return;
    };

    for (track_id, drop_zone) in drop_zones.iter() {
        if matches!(
            drop_zone.drop_phase(),
            DropPhase::DroppableEntered | DropPhase::DroppableHover
        ) {
            let Some(incoming) = drop_zone.incoming_droppable() else {
                continue;
            };
            if incoming != note_drag.note {
                continue;
            }

            let Ok((_note, old_track)) = notes.get(incoming) else {
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

            break;
        }
    }
}

// If no `DropZone` is active while a note is being dragged, move it back to its original track.
// (`DropZone` of the original track is not activated by dragging `Draggable` from the same track)
fn note_move_inactive(
    mut commands: Commands,
    notes: Query<(Entity, &Draggable, &Parent), With<Note>>,
    drop_zones: Query<&DropZone, With<Track>>,
    note_drag: Option<Res<NoteDragged>>,
) {
    let Some(note_drag) = note_drag else {
        return;
    };

    let all_inactive = drop_zones
        .iter()
        .all(|drop_zone| drop_zone.drop_phase() == DropPhase::Inactive);
    if all_inactive {
        let Ok((note_id, draggable, parent)) = notes.get(note_drag.note) else {
            error!(
                "NoteDragged points to a non-existent entity {:?}",
                note_drag.note
            );
            return;
        };

        if matches!(draggable.state, DragState::DragStart | DragState::Dragging) {
            if parent.get() != note_drag.orig_track {
                commands.entity(note_id).set_parent(note_drag.orig_track);
            }
        }
    }
}
