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

mod interaction;
mod notes;

use notes::*;

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

    app.add_systems(Update, update_seek_bar);

    app.add_plugins(interaction::plugin);
}

#[derive(Resource, Default)]
struct Sequencer {
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
    commands.init_resource::<Sequencer>();
    commands
        .ui_root_with_style(|style| Style {
            justify_content: JustifyContent::End,
            ..style
        })
        .insert(StateScoped(Screen::Playing))
        .with_children(|children| {
            children.label("Sequencer");

            children.spawn((
                Name::new("Seek Bar"),
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Px(5.0),
                        height: Percent(100.0),
                        ..default()
                    },
                    z_index: ZIndex::Local(999),
                    ..default()
                },
                BackgroundColor(BLUE_50.into()),
            ));
            // TODO: add seek bar handle

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
                });
        });
}

fn exit_playing(mut commands: Commands) {
    commands.remove_resource::<Sequencer>();
}

// What the play position, in seconds, will be if we seek fully from the left to the right.
const SEQUENCER_WIDTH_TIME: f32 = 8.0;

// Reflects the current play position on the seek bar.
fn update_seek_bar(mut seek_bar: Query<&mut Style, With<SeekBar>>, sequencer: Res<Sequencer>) {
    if !sequencer.is_changed() {
        return;
    }

    for mut style in seek_bar.iter_mut() {
        style.width = Percent(sequencer.play_pos / SEQUENCER_WIDTH_TIME * 100.0);
    }
}
