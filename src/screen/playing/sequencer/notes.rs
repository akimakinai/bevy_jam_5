use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, UiSystem, Val::*};
use bevy_debug_text_overlay::screen_print;
use sickle_ui_scaffold::prelude::{
    DragState, Draggable, DraggableUpdate, DropPhase, DropZone, Droppable, FluxInteraction,
    FluxInteractionUpdate, TrackedInteraction,
};
use std::ops::Deref;

use crate::ui::prelude::*;

use crate::screen::Screen;

use super::{Sequencer, Track, TRACK_WIDTH_TIME, TRACK_WIDTH};

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

    // When a note is added, set its initial position (convert time to pixels)
    app.add_systems(
        PostUpdate,
        set_initial_note_pos
            .run_if(in_state(Screen::Playing))
            .before(UiSystem::Layout),
    );
}

#[derive(Debug, Clone, Copy)]
pub enum NoteKind {
    Jump,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Note {
    pub kind: NoteKind,
    /// Position of this note in seconds, which is the time when this note should be played.
    // this will be applied to Style::left on `Added<Note>`,
    // and during dragging in [`super::interaction`] module.
    pub pos: f32,
    /// Width of this note in seconds.
    pub width: f32,
}

impl Note {
    pub const DEFAULT_WIDTH: f32 = 1.0;

    pub fn spawn(spawner: &mut impl Spawn, pos: f32) -> EntityCommands {
        spawner.spawn((
            TextBundle::from_section(
                "Drag me",
                TextStyle {
                    font_size: 24.0,
                    color: BLACK.into(),
                    ..default()
                },
            )
            .with_style(Style {
                width: Px(Note::DEFAULT_WIDTH * TRACK_WIDTH / TRACK_WIDTH_TIME),
                height: Percent(100.0),
                position_type: PositionType::Absolute,
                align_content: AlignContent::Center,
                ..default()
            })
            .with_background_color(WHITE.into()),
            Interaction::None,
            TrackedInteraction::default(),
            Draggable::default(),
            Droppable,
            RelativeCursorPosition::default(),
            Note {
                // TODO
                kind: NoteKind::Jump,
                pos,
                width: Note::DEFAULT_WIDTH,
            },
        ))
    }
}

// Run this in PostUpdate, because only UI logic is dependent of the appearent position of notes
fn set_initial_note_pos(mut added_notes: Query<(&Note, &mut Style), Added<Note>>) {
    for (note, mut style) in &mut added_notes {
        // A note is a child of a track, so left = 0 when pos = 0
        let ui_pos = note.pos / TRACK_WIDTH_TIME * TRACK_WIDTH;
        style.left = Px(ui_pos);
    }
}

// If a track is pressed and no notes are hovered, spawn a note at the cursor position.
fn track_interaction(
    mut commands: Commands,
    notes: Query<&Interaction, With<Note>>,
    tracks: Query<
        (Entity, &Name, &FluxInteraction, &RelativeCursorPosition),
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

    for (track_id, name, interaction, rel_cur_pos) in tracks.iter() {
        if interaction.is_pressed() {
            screen_print!("Track {:?} pressed", name);

            let cur_x = if let Some(touch_pos) = rel_cur_pos.normalized {
                touch_pos.x
            } else {
                // this could be unreachable!(), but let's be safe
                return;
            };

            commands.entity(track_id).with_children(|child| {
                // cur_x is relative to the size of the track
                // TODO: for a single click, spawn a note centered at the cursor,
                // and for a drag, spawn a note with the width of the drag.
                let pos_sec = cur_x * TRACK_WIDTH_TIME
                    - Note::DEFAULT_WIDTH * 0.5;
                let id = Note::spawn(child, pos_sec).id();
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
    mut sequencer: ResMut<Sequencer>,
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
                        if !(0.0..=TRACK_WIDTH).contains(&x) {
                            commands.entity(entity).despawn_recursive();
                            sequencer.notes.retain(|&id| id != entity);
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

        if matches!(draggable.state, DragState::DragStart | DragState::Dragging)
            && parent.get() != note_drag.orig_track
        {
            commands.entity(note_id).set_parent(note_drag.orig_track);
        }
    }
}
