// src/main.rs
mod tetris;

use std::f32::consts::PI;

use bevy::prelude::*;
use rand::Rng;
use tetris::{
    does_piece_fit, get_cells, spawn_tetromino, CurrentPiece, GameField, GameState, GameTimer,
    Score, Tetromino, CELL_SIZE, FIELD_HEIGHT, FIELD_WIDTH, TETROMINO_SHAPES,
};

// This system spawns the very first piece or can be called if CurrentPiece is None.
fn spawn_new_piece(
    mut commands: Commands,
    // current_piece_res: Option<ResMut<CurrentPiece>>,
    texture_square: Res<TextureSquareList>,
) {
    let mut rng = rand::thread_rng();
    let new_shape_index = rng.gen_range(0..TETROMINO_SHAPES.len());
    // let new_piece = CurrentPiece::new(new_shape_index);

    // if let Some(mut piece_res) = current_piece_res {
    //     // *piece_res = new_piece;
    //     println!(
    //         "Spawned piece (startup/manual, replacing existing): Index {}",
    //         new_shape_index
    //     );
    // } else {
    let sprite = Sprite::from_atlas_image(
        texture_square.texture.clone(),
        TextureAtlas {
            layout: texture_square.texture_atlas_layout.clone(),
            index: 0,
        },
    );
    let id = spawn_tetromino(&mut commands, sprite);
    commands.insert_resource(CurrentPiece { id });
    println!(
        "Spawned piece (startup/manual, inserting new): Index {}",
        new_shape_index
    );
    // }
}

#[derive(Resource)]
pub struct TextureSquareList {
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
}

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load::<Image>("textures/square-list.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 5, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        Camera2d::default(),
        Transform {
            translation: Vec3::new(
                (FIELD_WIDTH as f32 * CELL_SIZE as f32) / 2.0 - CELL_SIZE as f32,
                (FIELD_HEIGHT as f32 * CELL_SIZE as f32) / 2.0 - CELL_SIZE as f32,
                0.0,
            ),
            rotation: Quat::from_rotation_z(PI),
            ..default()
        },
    ));

    let game_field = GameField::new();
    let board_sprite = Sprite::from_atlas_image(
        texture.clone(),
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: 4,
        },
    );

    for y in 0..FIELD_HEIGHT {
        for x in 0..FIELD_WIDTH {
            if game_field.field[y * FIELD_WIDTH + x] == 9 {
                commands.spawn((
                    board_sprite.clone(),
                    Transform::from_xyz(
                        x as f32 * CELL_SIZE as f32,
                        y as f32 * CELL_SIZE as f32,
                        0.0,
                    ),
                ));
            }
        }
    }

    commands.insert_resource(game_field);
    commands.insert_resource(Score::default());
    commands.insert_resource(GameTimer::new(20));
    commands.insert_resource(TextureSquareList {
        texture: texture,
        texture_atlas_layout: texture_atlas_layout,
    });
    // let sprite = Sprite::from_atlas_image(
    //     texture,
    //     TextureAtlas {
    //         layout: texture_atlas_layout,
    //         index: 0,
    //     },
    // );

    // commands.spawn((
    //     Sprite::from_atlas_image(
    //         texture,
    //         TextureAtlas {
    //             layout: texture_atlas_layout,
    //             index: 0,
    //         },
    //     ),
    //     Transform::from_scale(Vec3::splat(1.0)),
    //     // animation_indices,
    //     // AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    // ));
    // spawn_tetromino(commands, sprite);

    println!("Game setup complete (core resources).");
}

fn player_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_piece_res: Option<ResMut<CurrentPiece>>,
    game_field: Res<GameField>,
    // mut tetromino: Query<(&mut Tetromino, &mut Transform, &Children)>,
    mut tetromino: Query<(Entity, &mut Tetromino, &Children)>,
    mut transform_q: Query<&mut Transform>,
) {
    if let Some(piece) = current_piece_res {
        let mut intended_dx: i32 = 0;
        let mut player_intended_dy = 0;
        let mut intended_rotation_change = false;

        // 由于camera旋转了180度
        // 需要把x操作反过来
        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            intended_dx += 1;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowRight) {
            intended_dx -= 1;
        }
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            player_intended_dy += 1;
        }
        if keyboard_input.just_pressed(KeyCode::KeyZ) {
            intended_rotation_change = true;
        }

        let id = piece.id;
        let (parent, mut piece, mut children) = tetromino.get_mut(id).unwrap();

        let mut transform = transform_q.get_mut(parent).unwrap();

        // 这里需要提前判断边界
        // 不然会因为u系列-1而越界噶嘣

        if intended_dx != 0 {
            // 换成i吧，有小于1的情况，比如竖条老哥可以跑到最右边应该是<0的情况
            if intended_dx < 0 {
                let sub_check = piece.position.x.checked_add_signed(intended_dx);
                if sub_check.is_none() {
                    // 越界了，直接返回不做任何操作
                    return;
                }
            }
            if does_piece_fit(
                &game_field,
                piece.shape_type,
                piece.rotation,
                (piece.position.x as i32 + intended_dx) as usize,
                piece.position.y as usize,
            ) {
                // println!("{}-{}", piece.position.x, transform.translation.x);
                piece.position.x = (piece.position.x as i32 + intended_dx) as u32;
                transform.translation.x += (intended_dx * CELL_SIZE as i32) as f32;
                // println!("a{}-{}", piece.position.x, transform.translation.x);
            }
        }
        if player_intended_dy != 0 {
            if does_piece_fit(
                &game_field,
                piece.shape_type,
                piece.rotation,
                piece.position.x as usize,
                (piece.position.y + player_intended_dy) as usize,
            ) {
                piece.position.y += player_intended_dy;
                transform.translation.y += (player_intended_dy * CELL_SIZE as u32) as f32;
            }
        }
        if intended_rotation_change {
            let new_rotation = (piece.rotation + 1) % 4;
            // const ROTATION: [f32; 4] = [0.0, PI / 2.0, PI, PI / 2.0 * 3.0];
            if does_piece_fit(
                &game_field,
                piece.shape_type,
                new_rotation,
                piece.position.x as usize,
                piece.position.y as usize,
            ) {
                piece.rotation = new_rotation;

                let cells = get_cells(piece.shape_type, new_rotation);
                // 不直接旋父节点了，既然字节点已经有旋转信息了
                // 可以直接更新子节点相对于父节点的位置，就是麻烦点=_=
                // 倒是对了，但嵌入了墙里
                let mut i = 0;
                for child in children {
                    if let Ok(mut transform) = transform_q.get_mut(*child) {
                        transform.translation.x = (cells[i].x * CELL_SIZE as u32) as f32;
                        transform.translation.y = (cells[i].y * CELL_SIZE as u32) as f32;
                        i += 1;
                    }
                }
            }
        }
    }
}

fn auto_fall_and_lock_system(
    time: Res<Time>,
    mut game_timer: ResMut<GameTimer>,
    current_piece_opt: Option<ResMut<CurrentPiece>>,
    mut game_field: ResMut<GameField>,
    mut score: ResMut<Score>,
    mut next_game_state: ResMut<NextState<GameState>>, // Added for state transition

    mut tetromino: Query<(&mut Tetromino, &mut Transform)>,
) {
    if let Some(piece) = current_piece_opt {
        game_timer.fall_timer.tick(time.delta());

        let mut force_down = false;
        if game_timer.fall_timer.just_finished() {
            force_down = true;
        }

        let id = piece.id;
        let mut piece = tetromino.get_mut(id).unwrap();

        if force_down {
            if does_piece_fit(
                &game_field,
                piece.0.shape_type,
                piece.0.rotation,
                piece.0.position.x as usize,
                (piece.0.position.y + 1) as usize,
            ) {
                piece.0.position.y += 1;
                piece.1.translation.y += CELL_SIZE as f32;
            } else {
                game_field.lock_piece(&piece.0);
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
                let shape_type = rng.gen_range(0..TETROMINO_SHAPES.len());
                // let new_piece_state = CurrentPiece::new(new_shape_index);
                let tetromino = Tetromino::new(shape_type);

                // x: (FIELD_WIDTH / 2) as i32 - 2, // Start roughly in the middle
                // y: 0,                            // Start at the top

                // *self.current_piece_res = new_piece_state;
                // println!("Respawned current piece: Index {}", new_shape_index);

                // respawn_current_piece();

                if !does_piece_fit(
                    &game_field,
                    tetromino.shape_type,
                    tetromino.rotation,
                    tetromino.position.x as usize,
                    tetromino.position.y as usize,
                ) {
                    println!("GAME OVER: New piece does not fit. Transitioning to GameOver state.");
                    next_game_state.set(GameState::GameOver); // Transition to GameOver
                }
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
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "tetirs".into(),
                resolution: (800.0, 600.0).into(),
                resizable: true,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_state::<GameState>()
        // .init_resource::<TextureSquareList>()
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
