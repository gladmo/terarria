use bevy::prelude::*;

use crate::player::SelectedBlock;
use crate::world::TileType;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(Update, update_selected_block_ui);
    }
}

/// Marker for the selected-block label.
#[derive(Component)]
struct SelectedBlockLabel;

fn spawn_ui(mut commands: Commands) {
    // Root node — fills the whole screen
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // Top bar: title / instructions
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new(
                            "WASD / Arrows: move   Space: jump\
                             \nLMB: mine   RMB: place   1-6: select block",
                        ),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Bottom bar: selected block
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Selected: Dirt"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        SelectedBlockLabel,
                    ));
                });
        });
}

fn update_selected_block_ui(
    selected: Res<SelectedBlock>,
    mut label_query: Query<&mut Text, With<SelectedBlockLabel>>,
) {
    if !selected.is_changed() {
        return;
    }
    let name = block_name(selected.0);
    for mut text in &mut label_query {
        text.0 = format!("Selected: {name}  [1-6 to change]");
    }
}

fn block_name(t: TileType) -> &'static str {
    match t {
        TileType::Air => "Air",
        TileType::Grass => "Grass",
        TileType::Dirt => "Dirt",
        TileType::Stone => "Stone",
        TileType::Bedrock => "Bedrock",
        TileType::Sand => "Sand",
        TileType::Wood => "Wood",
        TileType::Leaves => "Leaves",
    }
}
