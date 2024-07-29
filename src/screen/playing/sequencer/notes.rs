use bevy::color::palettes::css::{BLACK, WHITE};
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::ui::{RelativeCursorPosition, Val::*};
use sickle_ui_scaffold::prelude::{Draggable, Droppable, TrackedInteraction};

use crate::ui::prelude::*;

#[derive(Component)]
pub struct Note;

impl Note {
    pub const DEFAULT_WIDTH: f32 = 100.0;

    pub fn spawn(spawner: &mut impl Spawn, left_px: f32) -> EntityCommands {
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
                left: Px(left_px),
                width: Px(Note::DEFAULT_WIDTH),
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
        ))
    }
}
