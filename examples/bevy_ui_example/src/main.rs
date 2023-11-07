//! The [Bevy UI example] demonstrates a more interesting UI than just a couple of widgets. The
//! spawn logic is implemented as a monolithic system and while this works fine for an example, a
//! similar system in a real application with a larger UI will quickly become unruly. Let's see what
//! happens when we use `bevy_startup_tree` to break up the logic into more maintainble chunks.
//!
//! [Bevy UI example]: https://bevyengine.org/examples/ui/ui/

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    winit::WinitSettings,
};
use bevy_startup_tree::{startup_tree, AddStartupTree};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, spawn_camera)
        .add_startup_tree(startup_tree! {
            spawn_ui_containers => {
                spawn_left_panel_content,
                spawn_right_panel_content,
                spawn_middle_content,
            }
        })
        .add_systems(Update, mouse_scroll)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

#[derive(Component)]
struct RootNode;

#[derive(Component)]
struct LeftPanelNode;

#[derive(Component)]
struct RightPanelNode;

fn spawn_ui_containers(mut commands: Commands) {
    commands
        .spawn((
            RootNode,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            // left vertical fill (border)
            parent.spawn((
                LeftPanelNode,
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        height: Val::Percent(100.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                    ..default()
                },
            ));

            // right vertical fill
            parent.spawn((
                RightPanelNode,
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        width: Val::Px(200.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: Color::rgb(0.15, 0.15, 0.15).into(),
                    ..default()
                },
            ));
        });
}

fn spawn_left_panel_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_panel: Query<Entity, With<LeftPanelNode>>,
) {
    let panel_content_entity = commands
        .spawn(NodeBundle {
            style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..default()
        })
        .with_children(|parent| {
            // text
            parent.spawn(
                TextBundle::from_section(
                    "Text Example",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 30.0,
                        color: Color::WHITE,
                    },
                )
                .with_style(Style { margin: UiRect::all(Val::Px(5.0)), ..default() }),
            );
        })
        .id();
    commands.entity(q_panel.single()).add_child(panel_content_entity);
}

fn spawn_right_panel_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_panel: Query<Entity, With<RightPanelNode>>,
) {
    // Title
    let title_entity = commands
        .spawn(
            TextBundle::from_section(
                "Scrolling list",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 25.,
                    color: Color::WHITE,
                },
            )
            .with_style(Style {
                width: Val::Auto,
                height: Val::Px(25.0),
                margin: UiRect { left: Val::Auto, right: Val::Auto, ..default() },
                ..default()
            }),
        )
        .id();

    // List with hidden overflow
    let list_entity = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: Color::rgb(0.10, 0.10, 0.10).into(),
            ..default()
        })
        .with_children(|parent| {
            // Moving panel
            parent
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    },
                    ScrollingList::default(),
                ))
                .with_children(|parent| {
                    // List items
                    for i in 0..30 {
                        parent.spawn(
                            TextBundle::from_section(
                                format!("Item {i}"),
                                TextStyle {
                                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 20.,
                                    color: Color::WHITE,
                                },
                            )
                            .with_style(Style {
                                flex_shrink: 0.,
                                width: Val::Auto,
                                height: Val::Px(20.0),
                                margin: UiRect { left: Val::Auto, right: Val::Auto, ..default() },
                                ..default()
                            }),
                        );
                    }
                });
        })
        .id();

    commands.entity(q_panel.single()).push_children(&[title_entity, list_entity]);
}

fn spawn_middle_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_panel: Query<Entity, With<RootNode>>,
) {
    let blue_squares_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                left: Val::Px(210.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            background_color: Color::rgb(0.4, 0.4, 1.0).into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: Color::rgb(0.8, 0.8, 1.0).into(),
                ..default()
            });
        })
        .id();

    // render order test: reddest in the back, whitest in the front (flex center)
    let render_order_test_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style { width: Val::Px(100.0), height: Val::Px(100.0), ..default() },
                    background_color: Color::rgb(1.0, 0.0, 0.0).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(20.0),
                            bottom: Val::Px(20.0),
                            ..default()
                        },
                        background_color: Color::rgb(1.0, 0.3, 0.3).into(),
                        ..default()
                    });
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(40.0),
                            bottom: Val::Px(40.0),
                            ..default()
                        },
                        background_color: Color::rgb(1.0, 0.5, 0.5).into(),
                        ..default()
                    });
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(60.0),
                            bottom: Val::Px(60.0),
                            ..default()
                        },
                        background_color: Color::rgb(1.0, 0.7, 0.7).into(),
                        ..default()
                    });
                    // alpha test
                    parent.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(80.0),
                            bottom: Val::Px(80.0),
                            ..default()
                        },
                        background_color: Color::rgba(1.0, 0.9, 0.9, 0.4).into(),
                        ..default()
                    });
                });
        })
        .id();

    // bevy logo (flex center)
    let logo_entity = commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // bevy logo (image)
            parent.spawn(ImageBundle {
                style: Style { width: Val::Px(500.0), height: Val::Auto, ..default() },
                image: asset_server.load("branding/bevy_logo_dark_big.png").into(),
                ..default()
            });
        })
        .id();

    commands.entity(q_panel.single()).push_children(&[
        blue_squares_entity,
        render_order_test_entity,
        logo_entity,
    ]);
}

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Children, &Node)>,
    query_item: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, children, uinode) in &mut query_list {
            let items_height: f32 =
                children.iter().map(|entity| query_item.get(*entity).unwrap().size().y).sum();
            let panel_height = uinode.size().y;
            let max_scroll = (items_height - panel_height).max(0.);
            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}
