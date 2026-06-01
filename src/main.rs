use bevy::prelude::*;

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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(1.0, 1.0, 0.88)))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_grid)
        .run();
}

fn setup_grid(mut commands: Commands) {
    commands.spawn(Camera2d);

    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let x = (col as f32 - (GRID_COLUMNS as f32 - 1.0) / 2.0) * CELL_PITCH;
            let y = ((GRID_ROWS as f32 - 1.0) / 2.0 - row as f32) * CELL_PITCH;

            commands.spawn((
                Sprite::from_color(SQUARE_COLOR, Vec2::splat(SQUARE_SIZE)),
                Transform::from_xyz(x, y, 0.0),
            ));
        }
    }
}
