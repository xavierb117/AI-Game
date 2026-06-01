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
const VISITED_SQUARE_COLOR: Color = Color::srgb(0.98, 0.78, 0.78);
const BACKGROUND_COLOR: Color = Color::srgb(1.0, 1.0, 0.88);
const TOP_BAR_HEIGHT: f32 = 72.0;
const TOP_BAR_GAP_CM: f32 = 2.0;
const TOP_BAR_GAP: f32 = TOP_BAR_GAP_CM * PIXELS_PER_CM;
const TOP_BAR_FONT_SIZE: f32 = 42.0;
const SQUARE_FONT_SIZE: f32 = 34.0;
const SQUARE_NUMBER_MAX: i32 = 99;
const TARGET_MIN: i32 = 1;
const TARGET_MAX: i32 = 999;

const PLAYER_SIZE: f32 = 36.0;
const PLAYER_SPEED: f32 = 320.0;
const PLAYER_COLOR: Color = Color::srgb(0.15, 0.72, 0.28);
const CURRENT_SUM_FONT_SIZE: f32 = 42.0;
const CURRENT_SUM_PANEL_WIDTH: f32 = 140.0;
const GAME_TIMER_SECS: f32 = 180.0;
const TIMER_FONT_SIZE: f32 = 42.0;
const INSTRUCTIONS_PANEL_WIDTH: f32 = 280.0;
const INSTRUCTIONS_FONT_SIZE: f32 = 22.0;
const BARRIER_THICKNESS: f32 = 20.0;
const BARRIER_COLOR: Color = Color::srgb(0.82, 0.82, 0.82);
const GAME_OVER_OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.55);
const GAME_OVER_PANEL: Color = Color::srgb(1.0, 1.0, 1.0);
const TRY_AGAIN_BUTTON: Color = Color::srgb(0.25, 0.45, 0.85);
const TRY_AGAIN_BUTTON_HOVER: Color = Color::srgb(0.35, 0.55, 0.95);

const TOTAL_GRID_SQUARES: usize = (GRID_ROWS * GRID_COLUMNS) as usize;

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

#[derive(Component)]
struct Player;

/// Running total from grid squares the player has touched.
#[derive(Resource, Default)]
struct CurrentSum(i32);

#[derive(Component)]
struct CurrentSumUi;

#[derive(Component)]
struct TimerUi;

#[derive(Resource)]
struct GameTimer {
    remaining_secs: f32,
}

#[derive(Component)]
struct Visited;

/// Inner playable area inside the gray barriers (player stays within this).
#[derive(Resource, Clone, Copy)]
struct PlayAreaBounds {
    min: Vec2,
    max: Vec2,
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    Won,
    Lost,
}

#[derive(Component)]
struct GameEntity;

#[derive(Component)]
struct TargetUi;

#[derive(Component)]
struct EndGameScreen;

#[derive(Component)]
struct TryAgainButton;

fn main() {
    App::new()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                sync_target_display,
                (
                    move_player,
                    collect_square_values,
                    update_current_sum_display,
                    tick_timer,
                    update_timer_display,
                    check_game_end,
                    check_timer_expired,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
                try_again_button,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    setup_instructions(&mut commands);
    setup_current_sum_ui(&mut commands);
    setup_top_bar(&mut commands);
    start_new_round(&mut commands, &mut meshes, &mut materials);
}

fn start_new_round(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let (target, values) = generate_solvable_puzzle();
    commands.insert_resource(TargetNumber(target));
    commands.insert_resource(CurrentSum(0));
    commands.insert_resource(GameTimer::new(GAME_TIMER_SECS));

    let grid_offset_y = -((TOP_BAR_HEIGHT + TOP_BAR_GAP) / 2.0);
    commands.insert_resource(compute_play_area_bounds(grid_offset_y));

    setup_grid(commands, values, grid_offset_y);
    setup_barriers(commands, grid_offset_y);
    setup_player(commands, meshes, materials, grid_offset_y);
}

fn sync_target_display(target: Res<TargetNumber>, mut text: Single<&mut Text, With<TargetUi>>) {
    if target.is_changed() {
        **text = Text::new(target.0.to_string());
    }
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

fn setup_grid(
    commands: &mut Commands,
    values: [[i32; GRID_COLUMNS as usize]; GRID_ROWS as usize],
    grid_offset_y: f32,
) {
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLUMNS {
            let value = values[row as usize][col as usize];

            let x = (col as f32 - (GRID_COLUMNS as f32 - 1.0) / 2.0) * CELL_PITCH;
            let y = ((GRID_ROWS as f32 - 1.0) / 2.0 - row as f32) * CELL_PITCH + grid_offset_y;

            commands.spawn((
                Sprite::from_color(SQUARE_COLOR, Vec2::splat(SQUARE_SIZE)),
                Transform::from_xyz(x, y, 0.0),
                GameEntity,
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

fn compute_play_area_bounds(grid_offset_y: f32) -> PlayAreaBounds {
    let half_extent = 1.5 * CELL_PITCH + SQUARE_SIZE / 2.0;

    PlayAreaBounds {
        min: Vec2::new(-half_extent, grid_offset_y - half_extent),
        max: Vec2::new(half_extent, grid_offset_y + half_extent),
    }
}

fn setup_barriers(commands: &mut Commands, grid_offset_y: f32) {
    let bounds = compute_play_area_bounds(grid_offset_y);
    let grid_width = bounds.max.x - bounds.min.x;
    let grid_height = bounds.max.y - bounds.min.y;
    let center_x = (bounds.min.x + bounds.max.x) / 2.0;
    let center_y = (bounds.min.y + bounds.max.y) / 2.0;
    let wall_span_x = grid_width + 2.0 * BARRIER_THICKNESS;
    let wall_span_y = grid_height + 2.0 * BARRIER_THICKNESS;

    let barrier = |size: Vec2, position: Vec3| {
        (
            Sprite::from_color(BARRIER_COLOR, size),
            Transform::from_translation(position),
        )
    };

    commands.spawn((
        barrier(
            Vec2::new(wall_span_x, BARRIER_THICKNESS),
            Vec3::new(center_x, bounds.max.y + BARRIER_THICKNESS / 2.0, 5.0),
        ),
        GameEntity,
    ));
    commands.spawn((
        barrier(
            Vec2::new(wall_span_x, BARRIER_THICKNESS),
            Vec3::new(center_x, bounds.min.y - BARRIER_THICKNESS / 2.0, 5.0),
        ),
        GameEntity,
    ));
    commands.spawn((
        barrier(
            Vec2::new(BARRIER_THICKNESS, wall_span_y),
            Vec3::new(bounds.min.x - BARRIER_THICKNESS / 2.0, center_y, 5.0),
        ),
        GameEntity,
    ));
    commands.spawn((
        barrier(
            Vec2::new(BARRIER_THICKNESS, wall_span_y),
            Vec3::new(bounds.max.x + BARRIER_THICKNESS / 2.0, center_y, 5.0),
        ),
        GameEntity,
    ));
}

fn setup_player(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    grid_offset_y: f32,
) {
    let half = PLAYER_SIZE / 2.0;
    let triangle = Triangle2d::new(
        Vec2::new(0.0, half),
        Vec2::new(-half, -half),
        Vec2::new(half, -half),
    );

    commands.spawn((
        Mesh2d(meshes.add(triangle)),
        MeshMaterial2d(materials.add(PLAYER_COLOR)),
        Transform::from_xyz(0.0, grid_offset_y, 10.0),
        Player,
        GameEntity,
    ));
}

fn move_player(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    play_bounds: Res<PlayAreaBounds>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }

    if direction != Vec2::ZERO {
        direction = direction.normalize();
        transform.translation += (direction * PLAYER_SPEED * time.delta_secs()).extend(0.0);
    }

    let margin = PLAYER_SIZE / 2.0;
    transform.translation.x = transform
        .translation
        .x
        .clamp(play_bounds.min.x + margin, play_bounds.max.x - margin);
    transform.translation.y = transform
        .translation
        .y
        .clamp(play_bounds.min.y + margin, play_bounds.max.y - margin);
}

fn player_overlaps_square(player_pos: Vec2, square_pos: Vec2) -> bool {
    let half = SQUARE_SIZE / 2.0;
    let min = square_pos - Vec2::splat(half);
    let max = square_pos + Vec2::splat(half);

    player_pos.x >= min.x
        && player_pos.x <= max.x
        && player_pos.y >= min.y
        && player_pos.y <= max.y
}

fn collect_square_values(
    mut current_sum: ResMut<CurrentSum>,
    player: Query<&Transform, With<Player>>,
    mut squares: Query<(Entity, &Transform, &GridSquare, &mut Sprite), Without<Visited>>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, square_transform, grid_square, mut sprite) in &mut squares {
        let square_pos = square_transform.translation.truncate();
        if player_overlaps_square(player_pos, square_pos) {
            current_sum.0 += grid_square.value;
            sprite.color = VISITED_SQUARE_COLOR;
            commands.entity(entity).insert(Visited);
        }
    }
}

fn update_current_sum_display(
    current_sum: Res<CurrentSum>,
    text_root: Single<Entity, (With<CurrentSumUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*text_root, 1) = current_sum.0.to_string();
}

impl GameTimer {
    fn new(duration_secs: f32) -> Self {
        Self {
            remaining_secs: duration_secs,
        }
    }

    fn display_seconds(&self) -> u32 {
        self.remaining_secs.max(0.0).ceil() as u32
    }
}

fn tick_timer(time: Res<Time>, mut timer: ResMut<GameTimer>) {
    timer.remaining_secs -= time.delta_secs();
    if timer.remaining_secs < 0.0 {
        timer.remaining_secs = 0.0;
    }
}

fn update_timer_display(
    timer: Res<GameTimer>,
    text_root: Single<Entity, (With<TimerUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*text_root, 1) = timer.display_seconds().to_string();
}

fn check_timer_expired(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    timer: Res<GameTimer>,
    current_sum: Res<CurrentSum>,
    target: Res<TargetNumber>,
    end_screen: Query<(), With<EndGameScreen>>,
) {
    if timer.remaining_secs > 0.0 {
        return;
    }

    if current_sum.0 == target.0 as i32 {
        next_state.set(GameState::Won);
        if end_screen.is_empty() {
            spawn_end_screen(&mut commands, "You won!");
        }
        return;
    }

    next_state.set(GameState::Lost);
    if end_screen.is_empty() {
        spawn_end_screen(&mut commands, "You lost");
    }
}

fn check_game_end(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    current_sum: Res<CurrentSum>,
    target: Res<TargetNumber>,
    visited: Query<(), With<Visited>>,
    end_screen: Query<(), With<EndGameScreen>>,
) {
    if current_sum.0 == target.0 as i32 {
        next_state.set(GameState::Won);
        if end_screen.is_empty() {
            spawn_end_screen(&mut commands, "You won!");
        }
        return;
    }

    if visited.iter().count() < TOTAL_GRID_SQUARES {
        return;
    }

    next_state.set(GameState::Lost);
    if end_screen.is_empty() {
        spawn_end_screen(&mut commands, "You lost");
    }
}

fn spawn_end_screen(commands: &mut Commands, message: &str) {
    commands.spawn((
        EndGameScreen,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(GAME_OVER_OVERLAY),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(24),
                padding: UiRect::all(px(32)),
                border_radius: BorderRadius::all(px(12)),
                ..default()
            },
            BackgroundColor(GAME_OVER_PANEL),
            children![
                (
                    Text::new(message),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                ),
                (
                    Button,
                    TryAgainButton,
                    Node {
                        width: px(180),
                        height: px(56),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(px(8)),
                        ..default()
                    },
                    BackgroundColor(TRY_AGAIN_BUTTON),
                    children![(
                        Text::new("Try again"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    )],
                )
            ]
        )],
    ));
}

fn try_again_button(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<TryAgainButton>),
    >,
    game_entities: Query<Entity, With<GameEntity>>,
    end_screen: Query<Entity, With<EndGameScreen>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    state: Res<State<GameState>>,
) {
    if !matches!(*state.get(), GameState::Won | GameState::Lost) {
        return;
    }

    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                for entity in &game_entities {
                    commands.entity(entity).despawn();
                }
                for entity in &end_screen {
                    commands.entity(entity).despawn();
                }
                start_new_round(&mut commands, &mut meshes, &mut materials);
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => *color = TRY_AGAIN_BUTTON_HOVER.into(),
            Interaction::None => *color = TRY_AGAIN_BUTTON.into(),
        }
    }
}

fn setup_instructions(commands: &mut Commands) {
    let text_style = (
        TextFont {
            font_size: INSTRUCTIONS_FONT_SIZE,
            ..default()
        },
        TextColor(Color::BLACK),
    );

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::ZERO,
                top: px(TOP_BAR_HEIGHT + TOP_BAR_GAP),
                width: px(INSTRUCTIONS_PANEL_WIDTH),
                height: percent(100),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(20),
                ..default()
            },
        ))
        .with_children(|parent| {
            for line in [
                "Reach the target sum at the top before time runs out.",
                "Current count is the sum you are adding and substracting to",
                "Use arrow keys to move the triangle to the squares",
            ] {
                parent.spawn((Text::new(line), text_style.clone()));
            }
        });
}

fn setup_current_sum_ui(commands: &mut Commands) {
    commands.insert_resource(CurrentSum(0));
    commands.insert_resource(GameTimer::new(GAME_TIMER_SECS));

    let value_text_style = (
        TextFont {
            font_size: CURRENT_SUM_FONT_SIZE,
            ..default()
        },
        TextColor(Color::BLACK),
    );

    let timer_text_style = (
        TextFont {
            font_size: TIMER_FONT_SIZE,
            ..default()
        },
        TextColor(Color::BLACK),
    );

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::ZERO,
            top: px(TOP_BAR_HEIGHT + TOP_BAR_GAP),
            width: px(CURRENT_SUM_PANEL_WIDTH),
            height: percent(100),
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexEnd,
            row_gap: px(16),
            ..default()
        },
        children![
            (
                Text::new("Time"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ),
            (
                Text::new(""),
                TimerUi,
                timer_text_style.clone(),
                children![(
                    TextSpan::default(),
                    timer_text_style,
                )],
            ),
            (
                Text::new("Current"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ),
            (
                Text::new(""),
                CurrentSumUi,
                value_text_style.clone(),
                children![(TextSpan::default(), value_text_style)],
            )
        ],
    ));
}

fn setup_top_bar(commands: &mut Commands) {
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
                Text::new("0"),
                TargetUi,
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
