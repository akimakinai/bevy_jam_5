//! This module is responsible for the UI that displays the player's cards.
//!
//! Draggable `Card` entities are placed in slots.
//! A `Card` is positioned relative if it's in a slot, and absolute if it's being dragged.
//! Whatever this will look like (like a sequencer for DJ concept? Bot composer.), we can abstract it with cards and slots.
use crate::screen::Screen;
use crate::ui::prelude::*;
use bevy::color::palettes::tailwind::*;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};
use sickle_ui_scaffold::drag_interaction::{
    DragInteractionPlugin, DragState, Draggable, DraggableUpdate,
};
use sickle_ui_scaffold::drop_interaction::{DropInteractionPlugin, DropPhase, DropZone, Droppable};
use sickle_ui_scaffold::flux_interaction::{FluxInteractionPlugin, TrackedInteraction};

// This module is UI-only, and everything is under the UI root which has `StateScoped`.
// No other `spawn`s should need it.

pub fn plugin(app: &mut App) {
    app.add_plugins((
        FluxInteractionPlugin,
        DragInteractionPlugin,
        DropInteractionPlugin,
    ));

    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    // app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        (card_drag, card_drop)
            .run_if(in_state(Screen::Playing))
            .after(DraggableUpdate),
    );

    app.observe(handle_reset_position);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
enum Card {
    Forward,
    // Backward,
    // Jump,
    // Attack,
}

fn card_bundle(card: Card) -> impl Bundle {
    (
        NodeBundle {
            style: Style {
                width: Px(80.0),
                height: Px(80.0),
                ..default()
            },
            background_color: BLUE_300.into(),
            z_index: ZIndex::Global(999),
            ..default()
        },
        card,
        // Drag-and-drop components:
        Interaction::None,
        TrackedInteraction::default(),
        Draggable::default(),
        Droppable::default(),
        RelativeCursorPosition::default(),
    )
}

#[derive(Event)]
struct ResetPosition;

fn handle_reset_position(trigger: Trigger<ResetPosition>, mut styles: Query<&mut Style>) {
    let mut style = styles.get_mut(trigger.entity()).unwrap();

    if style.left != Px(0.) {
        style.left = Px(0.);
    }
    if style.top != Px(0.) {
        style.top = Px(0.);
    }
}

/// Update the position of the dragged card.
fn card_drag(
    mut commands: Commands,
    mut cards: Query<(Entity, &Draggable, &mut Style), (With<Card>, Changed<Draggable>)>,
) {
    for (entity, draggable, mut style) in cards.iter_mut() {
        debug!("{:?}", draggable);
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
                commands.trigger_targets(ResetPosition, entity);
            }
        }
    }
}

// #[derive(Event)]
// struct CardDrop(usize);

fn card_drop(
    mut commands: Commands,
    slots: Query<(&CardSlot, &DropZone), Changed<DropZone>>,
    cards: Query<&Card>,
    mut inventory: ResMut<Inventory>,
) {
    for (&CardSlot(slot), drop_zone) in slots.iter() {
        debug!("{:?} {:?}", slot, drop_zone);
        if drop_zone.drop_phase() != DropPhase::Dropped {
            continue;
        }
        let Some(card_id) = drop_zone.incoming_droppable() else {
            continue;
        };

        if inventory.cards[slot].is_some() {
            commands.trigger_targets(ResetPosition, card_id);
        }

        let Ok(&card) = cards.get(card_id) else {
            continue;
        };

        // Already occupied.
        if inventory.cards[slot].is_some() {
            commands.trigger_targets(ResetPosition, card_id);
            continue;
        }

        inventory.cards[slot] = Some(card);

        // FIXME: can't we change parent?
        commands.entity(card_id).despawn_recursive();

        commands
            .entity(inventory.slots[slot])
            .with_children(|children| {
                children.spawn(card_bundle(card));
            });
    }
}

const SLOTS_NUM: usize = 8;

#[derive(Component, Debug, Clone, Copy)]
struct CardSlot(usize);

#[derive(Resource)]
struct Inventory {
    slots: [Entity; SLOTS_NUM],
    // We can look up CardSlot's child, but using UI hierarchy as a source of truth is not robust.
    cards: [Option<Card>; SLOTS_NUM],
}

fn palette_empty() -> InteractionPalette {
    InteractionPalette {
        none: GRAY_300.into(),
        hovered: GRAY_400.into(),
        pressed: GRAY_500.into(),
    }
}

fn enter_playing(mut commands: Commands) {
    let mut slots = vec![];
    commands
        .ui_root_with_style(|style| Style {
            justify_content: JustifyContent::End,
            ..style
        })
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.label("Commands");
            children
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|children| {
                    for i in 0..SLOTS_NUM {
                        let id = children
                            .spawn((
                                NodeBundle {
                                    style: Style {
                                        width: Px(80.0),
                                        height: Px(80.0),
                                        margin: UiRect::all(Px(8.0)),
                                        border: UiRect::all(Px(3.0)),
                                        ..default()
                                    },
                                    border_color: ORANGE_600.into(),
                                    ..default()
                                },
                                CardSlot(i),
                                Interaction::None,
                                palette_empty(),
                                DropZone::default(),
                                RelativeCursorPosition::default(),
                            ))
                            .with_children(|children| {
                                // test
                                if i == 0 {
                                    children.spawn(card_bundle(Card::Forward));
                                }
                            })
                            .id();
                        slots.push(id);
                    }
                });
        });

    commands.insert_resource(Inventory {
        slots: slots.try_into().unwrap(),
        cards: default(),
    });
}
