//! A simplified implementation of the classic game "Picross".

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
// y coordinates
const BOTTOM_OFFSET: f32 = -200.;

const TILE_SIZE: Vec2 = Vec2::new(50., 50.);
// These values are exact
const GAP_BETWEEN_TILES: f32 = 5.0;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

const GRID_WIDTH: usize = 5;
const GRID_HEIGHT: usize = 5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (check_board, mouse_button_input))
        .run();
}

#[derive(Default,PartialEq)]
enum BlockState {
    #[default]
    Empty,
    Filled,
    Blocked
}

#[derive(Component, Default)]
struct Tile{
    state: BlockState,
}

#[derive(Component)]
struct HintTile;

#[derive(Component)]
struct MainCamera;

#[derive(Resource,Default)]
struct GameBoard {
    game_tiles: Vec<Entity>,
}

// Add the game's entities to our world
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn((Camera2dBundle::default(), MainCamera));

    let mut solution = GameSolution{ solution: vec![false; 25] };

    solution.solution[1] = true;
    solution.solution[2] = true;
    solution.solution[3] = true;
    solution.solution[5] = true;
    solution.solution[6] = true;
    solution.solution[7] = true;
    solution.solution[8] = true;
    solution.solution[9] = true;
    solution.solution[12] = true;
    solution.solution[17] = true;
    solution.solution[18] = true;
    solution.solution[22] = true;

    // Generate Hints

    let mut column_hints: Vec<Vec<usize>> = Vec::new();

    for column in 0..GRID_WIDTH {
        column_hints.push(Vec::new());
        let mut chain = 0;
        let mut hint_count = 0;
        for row in 0..GRID_HEIGHT{
            if solution.solution[row*GRID_WIDTH + column]{
                chain+=1;
            }else{
                if chain != 0{
                    println!("Hint for column {}: {}", column, chain);
                    column_hints[column].push(chain);
                    hint_count += 1;
                    chain = 0;
                }
            }
        }
        if chain != 0 || hint_count == 0{
            println!("Hint for column {}: {}", column, chain);
            column_hints[column].push(chain);
        }
    }

    let mut row_hints: Vec<Vec<usize>> = Vec::new();
    for row in 0..GRID_HEIGHT {
        row_hints.push(Vec::new());
        let mut chain = 0;
        let mut hint_count = 0;
        for column in 0..GRID_HEIGHT{
            if solution.solution[row*GRID_WIDTH + column]{
                chain+=1;
            }else{
                if chain != 0{
                    println!("Hint for row {}: {}", row, chain);
                    row_hints[row].push(chain);
                    hint_count += 1;
                    chain = 0;
                }
            }
        }
        if chain != 0 || hint_count == 0{
            println!("Hint for row {}: {}", row, chain);
            row_hints[row].push(chain);
        }
    }

    commands.insert_resource(solution);

    let bottom_edge_of_tiles = BOTTOM_OFFSET;

    let n_columns = GRID_WIDTH;
    let n_rows = GRID_HEIGHT;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the tiles only captures a lower bound, not an exact value
    let center_of_tiles: f32 = 0.0;
    let left_edge_of_tiles = center_of_tiles
        // Space taken up by the tiles
        - (n_columns as f32 / 2.0 * TILE_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_TILES;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_tiles + TILE_SIZE.x / 2.;
    let offset_y = bottom_edge_of_tiles + TILE_SIZE.y / 2.;

    let mut controller = GameBoard::default();

    for row in 0..n_rows {
        for column in 0..n_columns {
            let tile_position = Vec2::new(
                offset_x + column as f32 * (TILE_SIZE.x + GAP_BETWEEN_TILES),
                offset_y + row as f32 * (TILE_SIZE.y + GAP_BETWEEN_TILES),
            );


            // tile
            let asdf = commands.spawn((
                SpriteBundle {
                    texture: asset_server.load("textures/empty.png"),

                    transform: Transform {
                        translation: tile_position.extend(0.0),
                        //scale: Vec3::new(TILE_SIZE.x, TILE_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Tile::default(),
            )).id();
            controller.game_tiles.push(asdf);
        }
    }

    // Create column hint tiles
    for column_id in 0..column_hints.len(){
        let hint_list_len = column_hints[column_id].len();
        for hint_id in 0..hint_list_len{
            let column = column_id;
            let row = GRID_HEIGHT + hint_id;
            let tile_position = Vec2::new(
                offset_x + column as f32 * (TILE_SIZE.x + GAP_BETWEEN_TILES),
                offset_y + row as f32 * (TILE_SIZE.y + GAP_BETWEEN_TILES),
            );


            // Hint tile
            let hint_tex = format!("textures/{}.png", column_hints[column_id][hint_id]);
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(hint_tex),
                    transform: Transform {
                        translation: tile_position.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                HintTile,
            ));

        }
    }

    // Create row hint tiles
    for row_id in 0..row_hints.len(){
        let hint_list_len = row_hints[row_id].len();
        for hint_id in 0..hint_list_len{
            let column: isize = -1 - hint_id as isize;
            let row = 0 + row_id;
            let tile_position = Vec2::new(
                offset_x + column as f32 * (TILE_SIZE.x + GAP_BETWEEN_TILES),
                offset_y + row as f32 * (TILE_SIZE.y + GAP_BETWEEN_TILES),
            );

            let hint_tex = format!("textures/{}.png", row_hints[row_id][hint_id]);

            // Hint tile
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(hint_tex),
                    transform: Transform {
                        translation: tile_position.extend(0.0),
                        ..default()
                    },
                    ..default()
                },
                HintTile,
            ));
        }
    }
    commands.insert_resource(controller);

}

fn mouse_button_input(
    buttons: Res<Input<MouseButton>>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    // tile data
    mut tiles: Query<(&mut Sprite, &Transform, &mut Tile)>,
) {
        if buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right){
            // get the camera info and transform
            // assuming there is exactly one main camera entity, so Query::single() is OK
            let (camera, camera_transform) = q_camera.single();

            // There is only one primary window, so we can similarly get it from the query:
            let window = q_window.single();

            // check if the cursor is inside the window and get its position
            // then, ask bevy to convert into world coordinates, and truncate to discard Z
            let world_position = window.cursor_position()
                .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                .map(|ray| ray.origin.truncate()).unwrap();

            for (mut s, t, mut tile) in tiles.iter_mut(){
            // Copilot autocompleted this so IDK if it actually works
            if t.translation.x - TILE_SIZE.x/2. < world_position.x && world_position.x < t.translation.x + TILE_SIZE.x/2. && t.translation.y - TILE_SIZE.y/2. < world_position.y && world_position.y < t.translation.y + TILE_SIZE.y/2. {
                if buttons.just_pressed(MouseButton::Left){
                    if tile.state == BlockState::Filled{
                        tile.state = BlockState::Empty;
                        s.color = Color::WHITE;
                    }
                    else if tile.state == BlockState::Blocked{
                        return;
                    }
                    else if tile.state == BlockState::Empty{
                        tile.state = BlockState::Filled;
                        s.color = Color::GRAY;
                    }
                } else if buttons.just_pressed(MouseButton::Right){
                    if tile.state == BlockState::Filled {
                        return;
                    }
                    else if tile.state == BlockState::Blocked {
                        tile.state = BlockState::Empty;
                        s.color = Color::WHITE;
                    }
                    else if tile.state == BlockState::Empty {
                        tile.state = BlockState::Blocked;
                        s.color = Color::ORANGE_RED;
                    }
                }
            }
        }
    }
}

#[derive(Resource, Default)]
struct GameSolution{
    solution: Vec<bool>,
}

fn check_board(
    game_state: ResMut<GameBoard>,
    solution: Res<GameSolution>,
    tile_query: Query<&Tile>,
){
    for (tile, solution_tile) in game_state.game_tiles.iter().zip(solution.solution.iter()){
        let wow = tile_query.get(*tile).unwrap();
        if (*solution_tile && !(wow.state == BlockState::Filled)) || (!(*solution_tile) && (wow.state == BlockState::Filled)){
            return;
        }
        
    }
    println!("Solution found!");
}