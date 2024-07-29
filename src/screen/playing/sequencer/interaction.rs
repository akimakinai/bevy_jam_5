use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};
use bevy_debug_text_overlay::screen_print;
use sickle_ui_scaffold::prelude::{
    DragState, Draggable, DraggableUpdate, DropPhase, DropZone, FluxInteraction,
    FluxInteractionUpdate,
};
use std::ops::Deref;

use crate::screen::Screen;

use super::{Note, Sequencer, Track, TRACK_WIDTH};

pub fn plugin(app: &mut App) {
    app.register_type::<NoteDragged>();
    // #[cfg(feature = "dev")]
    // app.add_plugins(ResourceInspectorPlugin::<NoteDragged>::default());

    // Handles adding notes
    app.add_systems(
        Update,
        track_interaction
            .after(FluxInteractionUpdate)
            .run_if(in_state(Screen::Playing)),
    );

    // Handles dragging notes
    app.add_systems(
        Update,
        (
            note_drag,
            (note_move_between_tracks, note_move_inactive).run_if(resource_exists::<NoteDragged>),
        )
            .chain()
            .run_if(in_state(Screen::Playing))
            .after(DraggableUpdate),
    );
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
    mut sequencer: ResMut<Sequencer>,
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
                let id = Note::spawn(child, track_width * cur_x - Note::DEFAULT_WIDTH / 2.0).id();
                sequencer.notes.push(id);
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
    mut notes: Query<(Entity, &Draggable, &Parent, &mut Style), (With<Note>, Changed<Draggable>)>,
    note_drag: Option<ResMut<NoteDragged>>,
) {
    for (entity, draggable, parent, mut style) in notes.iter_mut() {
        if matches!(
            draggable.state,
            DragState::DragEnd | DragState::DragCanceled | DragState::Inactive
        ) {
            if let Some(note_drag) = &note_drag {
                if note_drag.note == entity {
                    commands.remove_resource::<NoteDragged>();

                    // If the note is dropped outside of the track, remove it
                    if let Px(x) = style.left {
                        if x < 0.0 || x > TRACK_WIDTH {
                            commands.entity(entity).despawn_recursive();
                            // TODO: play sfx when note removed
                        }
                    }
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
    note_drag: Res<NoteDragged>,
) {
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
    note_drag: Res<NoteDragged>,
) {
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
