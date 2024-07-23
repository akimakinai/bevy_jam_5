#![allow(unused)]

use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};

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
        // DropInteractionPlugin,
    ));

    app.add_systems(OnEnter(Screen::Playing), enter_playing);

    app.add_systems(
        Update,
        note_drag
            .run_if(in_state(Screen::Playing))
            .after(DraggableUpdate),
    );
}

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
                    for s in ["Jump", "Attack", "Wait", "booh"] {
                        children.spawn(TextBundle::from_section(
                            s,
                            TextStyle {
                                font_size: 24.0,
                                color: WHITE.into(),
                                ..default()
                            },
                        ));

                        children
                            .spawn(NodeBundle {
                                style: Style {
                                    width: Px(500.0),
                                    height: Px(50.0),
                                    border: UiRect::all(Px(3.0)),
                                    ..default()
                                },
                                border_color: BLUE_200.into(),
                                ..default()
                            })
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

/// Update the position of the dragged card.
fn note_drag(
    mut commands: Commands,
    mut cards: Query<(Entity, &Draggable, &mut Style), (With<Note>, Changed<Draggable>)>,
) {
    for (entity, draggable, mut style) in cards.iter_mut() {
        // FIXME: can't we use absolute positioning for the dragged card?
        match draggable.state {
            DragState::DragStart | DragState::Dragging => {
                if let (Some(origin), Some(position)) = (draggable.origin, draggable.position) {
                    let diff = position - origin;
                    style.left = Px(diff.x);
                    style.top = Px(diff.y);
                }
            }
            DragState::DragEnd => {
                // Position will be set by the drop interaction.
            }
            _ => {
                if style.left != Px(0.) {
                    style.left = Px(0.);
                }
                if style.top != Px(0.) {
                    style.top = Px(0.);
                }
            }
        }
    }
}
