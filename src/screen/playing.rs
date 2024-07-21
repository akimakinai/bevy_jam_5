//! The screen state for the main game loop.

use bevy::color::palettes::tailwind::*;
use bevy::ui::Val::*;
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use super::Screen;
use crate::game::spawn::level::SpawnLevel;
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Playing), enter_playing);
    app.add_systems(OnExit(Screen::Playing), exit_playing);

    app.add_systems(
        Update,
        return_to_title_screen
            .run_if(in_state(Screen::Playing).and_then(input_just_pressed(KeyCode::Escape))),
    );
}

fn enter_playing(mut commands: Commands, mut proj: Query<&mut OrthographicProjection>) {
    commands.trigger(SpawnLevel);

    commands
        .spawn((
            Name::new("UI Root"),
            NodeBundle {
                style: Style {
                    width: Percent(100.0),
                    height: Percent(100.0),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Px(10.0),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
        ))
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
                    for _ in 0..8 {
                        children.spawn(NodeBundle {
                            style: Style {
                                width: Px(80.0),
                                height: Px(80.0),
                                margin: UiRect::all(Px(8.0)),
                                border: UiRect::all(Px(3.0)),
                                ..default()
                            },
                            border_color: ORANGE_600.into(),
                            ..default()
                        });
                    }
                });
        });

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
