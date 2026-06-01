use std::collections::HashSet;

use bevy::prelude::*;
use rand::{seq::IndexedRandom, seq::SliceRandom, RngExt};

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
const SQUARE_FONT_SIZE: f32 = 34.0;
const SQUARE_NUMBER_MAX: i32 = 99;
const TARGET_MIN: i32 = 1;
const TARGET_MAX: i32 = 999;

/// Random value chosen at startup; use this resource for game logic later.
#[derive(Resource, Debug, Clone, Copy)]
struct TargetNumber(u32);

/// Values shown in each grid cell, indexed by `[row][col]`.
#[derive(Resource, Clone, Copy)]
#[allow(dead_code)]
struct GridValues([[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize]);

#[derive(Component, Clone, Copy)]
#[allow(dead_code)]
struct GridSquare {
    row: usize,
    col: usize,
    value: i32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    let (target, values) = generate_solvable_puzzle();
    commands.insert_resource(TargetNumber(target));

    commands.spawn(Camera2d);
    setup_grid(&mut commands, values);
    setup_top_bar(&mut commands, target);
}

fn generate_grid_values() -> [[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize] {
    let cell_count = (GRID_ROWS * GRID_COLUMNS) as usize;
    let positive_count = cell_count / 2;

    let mut signs = vec![true; positive_count];
    signs.extend(vec![false; cell_count - positive_count]);
    signs.shuffle(&mut rand::rng());

    let mut values = [[0; GRID_COLUMNS as usize]; GRID_ROWS as usize];
    let mut sign_index = 0;

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let magnitude = rand::rng().random_range(1..=SQUARE_NUMBER_MAX);
            values[row as usize][col as usize] = if signs[sign_index] {
                magnitude
            } else {
                -magnitude
            };
            sign_index += 1;
        }
    }

    values
}

/// Every sum achievable by choosing any subset of grid cells (including none → 0).
fn subset_sums(values: &[[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize]) -> HashSet<i32> {
    let mut sums = HashSet::from([0]);

    for row in values {
        for &value in row {
            let extensions: Vec<i32> = sums.iter().map(|sum| sum + value).collect();
            sums.extend(extensions);
        }
    }

    sums
}

/// Build a random grid and pick a target that some subset of its cells can sum to.
fn generate_solvable_puzzle() -> (u32, [[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize]) {
    loop {
        let values = generate_grid_values();
        let valid_targets: Vec<i32> = subset_sums(&values)
            .into_iter()
            .filter(|sum| *sum >= TARGET_MIN && *sum <= TARGET_MAX)
            .collect();

        if let Some(target) = valid_targets.choose(&mut rand::rng()) {
            return (*target as u32, values);
        }
    }
}

fn setup_grid(commands: &mut Commands, values: [[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize]) {
    let grid_offset_y = -((TOP_BAR_HEIGHT + TOP_BAR_GAP) / 2.0);

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let value = values[row as usize][col as usize];

            let x = (col as f32 - (GRID_COLUMNS as f32 - 1.0) / 2.0) * CELL_PITCH;
            let y = ((GRID_ROWS as f32 - 1.0) / 2.0 - row as f32) * CELL_PITCH + grid_offset_y;

            commands.spawn((
                Sprite::from_color(SQUARE_COLOR, Vec2::splat(SQUARE_SIZE)),
                Transform::from_xyz(x, y, 0.0),
                GridSquare {
                    row: row as usize,
                    col: col as usize,
                    value,
                },
                children![(
                    Text2d::new(value.to_string()),
                    TextFont {
                        font_size: SQUARE_FONT_SIZE,
                        ..default()
                    },
                    TextLayout::new_with_justify(Justify::Center),
                    TextColor(Color::BLACK),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                )],
            ));
        }
    }

    commands.insert_resource(GridValues(values));
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
