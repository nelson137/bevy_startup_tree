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
    commands.spawn(Camera2d);
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
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            // left vertical fill (border)
            parent.spawn((
                LeftPanelNode,
                BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
                Node {
                    width: Val::Px(200.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
            ));

            // right vertical fill
            parent.spawn((
                RightPanelNode,
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                Node {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    width: Val::Px(200.0),
                    height: Val::Percent(100.0),
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
        .spawn((
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
        ))
        .with_children(|parent| {
            // text
            parent.spawn((
                Text::new("Text Example"),
                TextColor(Color::WHITE),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 30.0,
                    ..default()
                },
                Node { margin: UiRect::all(Val::Px(5.0)), ..default() },
            ));
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
        .spawn((
            Text::new("Scrolling list"),
            TextColor(Color::WHITE),
            TextFont {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 25.,
                ..default()
            },
            Node {
                width: Val::Auto,
                height: Val::Px(25.0),
                margin: UiRect { left: Val::Auto, right: Val::Auto, ..default() },
                ..default()
            },
        ))
        .id();

    // List with hidden overflow
    let list_entity = commands
        .spawn((
            ScrollingListViewport,
            BackgroundColor(Color::srgb(0.10, 0.10, 0.10)),
            Node {
                flex_direction: FlexDirection::Column,
                align_self: AlignSelf::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(50.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Moving panel
            parent
                .spawn((
                    Node { flex_direction: FlexDirection::Column, flex_grow: 1.0, ..default() },
                    ScrollingList::default(),
                ))
                .with_children(|parent| {
                    // List items
                    for i in 0..30 {
                        parent.spawn((
                            Text::new(format!("Item {i}")),
                            TextColor(Color::WHITE),
                            TextFont {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.,
                                ..default()
                            },
                            Node {
                                flex_shrink: 0.,
                                width: Val::Auto,
                                height: Val::Px(20.0),
                                margin: UiRect { left: Val::Auto, right: Val::Auto, ..default() },
                                ..default()
                            },
                        ));
                    }
                });
        })
        .id();

    commands.entity(q_panel.single()).add_children(&[title_entity, list_entity]);
}

fn spawn_middle_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_panel: Query<Entity, With<RootNode>>,
) {
    let blue_squares_entity = commands
        .spawn((
            BackgroundColor(Color::srgb(0.4, 0.4, 1.0)),
            Node {
                width: Val::Px(200.0),
                height: Val::Px(200.0),
                position_type: PositionType::Absolute,
                left: Val::Px(210.0),
                bottom: Val::Px(10.0),
                border: UiRect::all(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                BackgroundColor(Color::srgb(0.8, 0.8, 1.0)),
                Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            ));
        })
        .id();

    // render order test: reddest in the back, whitest in the front (flex center)
    let render_order_test_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
                    Node { width: Val::Px(100.0), height: Val::Px(100.0), ..default() },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        BackgroundColor(Color::srgb(1.0, 0.3, 0.3)),
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(20.0),
                            bottom: Val::Px(20.0),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        BackgroundColor(Color::srgb(1.0, 0.5, 0.5)),
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(40.0),
                            bottom: Val::Px(40.0),
                            ..default()
                        },
                    ));
                    parent.spawn((
                        BackgroundColor(Color::srgb(1.0, 0.7, 0.7)),
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(60.0),
                            bottom: Val::Px(60.0),
                            ..default()
                        },
                    ));
                    // alpha test
                    parent.spawn((
                        BackgroundColor(Color::srgba(1.0, 0.9, 0.9, 0.4)),
                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(100.0),
                            position_type: PositionType::Absolute,
                            left: Val::Px(80.0),
                            bottom: Val::Px(80.0),
                            ..default()
                        },
                    ));
                });
        })
        .id();

    // bevy logo (flex center)
    let logo_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|parent| {
            // bevy logo (image)
            parent.spawn((
                ImageNode::new(asset_server.load("branding/bevy_logo_dark_big.png")),
                Node { width: Val::Px(500.0), height: Val::Auto, ..default() },
            ));
        })
        .id();

    commands.entity(q_panel.single()).add_children(&[
        blue_squares_entity,
        render_order_test_entity,
        logo_entity,
    ]);
}

#[derive(Component)]
struct ScrollingListViewport;

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    query_list_viewport: Query<&ComputedNode, With<ScrollingListViewport>>,
    mut query_list: Query<(&mut ScrollingList, &mut Node, &Children)>,
    query_item: Query<&ComputedNode>,
) {
    let viewport_height = query_list_viewport.single().size().y;
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut node, children) in &mut query_list {
            let items_height: f32 =
                children.iter().map(|entity| query_item.get(*entity).unwrap().size().y).sum();
            let max_scroll = (items_height - viewport_height).max(0.);
            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            node.top = Val::Px(scrolling_list.position);
        }
    }
}
