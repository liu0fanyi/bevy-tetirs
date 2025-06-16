// src/main.rs
mod tetris;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*; // Required for SystemParam
                      // use bevy::prelude::NextState; // Included in prelude, but explicit for clarity if preferred
use rand::Rng;
use tetris::{
    does_piece_fit, CurrentPiece, GameField, GameState, GameTimer, Score, TETROMINO_SHAPES,
};

// This system spawns the very first piece or can be called if CurrentPiece is None.
fn spawn_new_piece(mut commands: Commands, current_piece_res: Option<ResMut<CurrentPiece>>) {
    let mut rng = rand::thread_rng();
    let new_shape_index = rng.gen_range(0..TETROMINO_SHAPES.len());
    let new_piece = CurrentPiece::new(new_shape_index);

    if let Some(mut piece_res) = current_piece_res {
        *piece_res = new_piece;
        println!(
            "Spawned piece (startup/manual, replacing existing): Index {}",
            new_shape_index
        );
    } else {
        commands.insert_resource(new_piece);
        println!(
            "Spawned piece (startup/manual, inserting new): Index {}",
            new_shape_index
        );
    }
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load::<Image>("textures/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);

    commands.spawn(Camera2d::default());

    commands.insert_resource(GameField::new());
    commands.insert_resource(Score::default());
    commands.insert_resource(GameTimer::new(20));
    println!("Game setup complete (core resources).");
}

fn player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_piece_res: Option<ResMut<CurrentPiece>>,
    game_field: Res<GameField>,
) {
    if let Some(mut piece) = current_piece_res {
        let mut intended_dx = 0;
        let mut player_intended_dy = 0;
        let mut intended_rotation_change = false;

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            intended_dx -= 1;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            intended_dx += 1;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            player_intended_dy += 1;
        }
        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            intended_rotation_change = true;
        }

        if intended_dx != 0 {
            if does_piece_fit(
                &game_field,
                piece.shape_index,
                piece.rotation,
                piece.x + intended_dx,
                piece.y,
            ) {
                piece.x += intended_dx;
            }
        }
        if player_intended_dy != 0 {
            if does_piece_fit(
                &game_field,
                piece.shape_index,
                piece.rotation,
                piece.x,
                piece.y + player_intended_dy,
            ) {
                piece.y += player_intended_dy;
            }
        }
        if intended_rotation_change {
            let new_rotation = (piece.rotation + 1) % 4;
            if does_piece_fit(
                &game_field,
                piece.shape_index,
                new_rotation,
                piece.x,
                piece.y,
            ) {
                piece.rotation = new_rotation;
            }
        }
    }
}

// #[derive(SystemParam)]
// pub struct InGamePieceSpawner<'w> {
//     current_piece_res: ResMut<'w, CurrentPiece>,
// }

// impl<'w> InGamePieceSpawner<'w> {
//     pub fn respawn_current_piece(&mut self) {
//         let mut rng = rand::thread_rng();
//         let new_shape_index = rng.gen_range(0..TETROMINO_SHAPES.len());
//         let new_piece_state = CurrentPiece::new(new_shape_index);

//         *self.current_piece_res = new_piece_state;
//         println!("Respawned current piece: Index {}", new_shape_index);
//     }
// }

fn auto_fall_and_lock_system(
    mut commands: Commands,
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    current_piece_opt: Option<ResMut<CurrentPiece>>,
    mut game_field: ResMut<GameField>,
    mut score: ResMut<Score>,
    // mut spawner: InGamePieceSpawner,
    mut next_game_state: ResMut<NextState<GameState>>, // Added for state transition
) {
    if let Some(mut piece) = current_piece_opt {
        game_timer.fall_timer.tick(time.delta());

        let mut force_down = false;
        if game_timer.fall_timer.just_finished() {
            force_down = true;
        }

        if force_down {
            if does_piece_fit(
                &game_field,
                piece.shape_index,
                piece.rotation,
                piece.x,
                piece.y + 1,
            ) {
                piece.y += 1;
            } else {
                game_field.lock_piece(&piece);
                score.0 += 25;
                println!(
                    "Piece locked. Base score added. Current Score: {}.",
                    score.0
                );

                let lines_cleared = game_field.check_and_clear_lines();
                if lines_cleared > 0 {
                    let line_clear_score = (1 << lines_cleared) * 100;
                    score.0 += line_clear_score;
                    println!(
                        "Lines cleared: {}. Additional score: {}. Total Score: {}",
                        lines_cleared, line_clear_score, score.0
                    );
                }

                let mut rng = rand::thread_rng();
                let new_shape_index = rng.gen_range(0..TETROMINO_SHAPES.len());
                let new_piece_state = CurrentPiece::new(new_shape_index);

                *self.current_piece_res = new_piece_state;
                println!("Respawned current piece: Index {}", new_shape_index);

                // spawner.respawn_current_piece();

                // if !does_piece_fit(
                //     &game_field,
                //     spawner.current_piece_res.shape_index,
                //     spawner.current_piece_res.rotation,
                //     spawner.current_piece_res.x,
                //     spawner.current_piece_res.y,
                // ) {
                //     println!("GAME OVER: New piece does not fit. Transitioning to GameOver state.");
                //     next_game_state.set(GameState::GameOver); // Transition to GameOver
                // }
            }
        }
    }
}

fn setup_game_over_screen(mut commands: Commands) {
    println!("Game Over! Entered GameState::GameOver.");
    // Example of spawning UI elements could go here
}

fn cleanup_game_over_screen() {
    println!("Exiting GameState::GameOver (e.g., if restarting).");
    // Despawn UI elements specific to game over screen
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, (setup_game, spawn_new_piece).chain())
        .add_systems(
            Update,
            (player_input_system, auto_fall_and_lock_system)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), setup_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), cleanup_game_over_screen)
        .run();
}
