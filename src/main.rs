use bevy::prelude::*;
use rand::RngExt;

/// Windows default display DPI: one inch ≈ this many pixels in 2D world space.
const PIXELS_PER_INCH: f32 = 96.0;
const CM_PER_INCH: f32 = 2.54;
const PIXELS_PER_CM: f32 = PIXELS_PER_INCH / CM_PER_INCH;

const GRID_COLUMNS: u32 = 4;
const GRID_ROWS: u32 = 4;
const GAP_CM: f32 = 2.0;
const SQUARE_INCHES: f32 = 1.0;

const SQUARE_SIZE: f32 = SQUARE_INCHES * PIXELS_PER_INCH;
const GAP_SIZE: f32 = GAP_CM * PIXELS_PER_CM;
const CELL_PITCH: f32 = SQUARE_SIZE + GAP_SIZE;

const SQUARE_COLOR: Color = Color::srgb(0.68, 0.85, 0.98);
const BACKGROUND_COLOR: Color = Color::srgb(1.0, 1.0, 0.88);
const TOP_BAR_HEIGHT: f32 = 72.0;
const TOP_BAR_GAP_CM: f32 = 2.0;
const TOP_BAR_GAP: f32 = TOP_BAR_GAP_CM * PIXELS_PER_CM;
const TOP_BAR_FONT_SIZE: f32 = 42.0;

/// Random value chosen at startup; use this resource for game logic later.
#[derive(Resource, Debug, Clone, Copy)]
struct TargetNumber(u32);

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let target = TargetNumber(random_target_number());
    commands.insert_resource(target);

    commands.spawn(Camera2d);
    setup_grid(&mut commands);
    setup_top_bar(&mut commands, target.0);
}

fn random_target_number() -> u32 {
    rand::rng().random_range(1..=999)
}

fn setup_grid(commands: &mut Commands) {
    let grid_offset_y = -((TOP_BAR_HEIGHT + TOP_BAR_GAP) / 2.0);

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let x = (col as f32 - (GRID_COLUMNS as f32 - 1.0) / 2.0) * CELL_PITCH;
            let y = ((GRID_ROWS as f32 - 1.0) / 2.0 - row as f32) * CELL_PITCH + grid_offset_y;

            commands.spawn((
                Sprite::from_color(SQUARE_COLOR, Vec2::splat(SQUARE_SIZE)),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }
}

fn setup_top_bar(commands: &mut Commands, number: u32) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                left: Val::ZERO,
                width: percent(100),
                height: px(TOP_BAR_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(SQUARE_COLOR),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(number.to_string()),
                TextFont {
                    font_size: TOP_BAR_FONT_SIZE,
                    ..default()
                },
                TextColor(Color::BLACK),
            ));
        });

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: px(TOP_BAR_HEIGHT),
            left: Val::ZERO,
            width: percent(100),
            height: px(TOP_BAR_GAP),
            ..default()
        },
        BackgroundColor(BACKGROUND_COLOR),
    ));
}
