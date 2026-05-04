use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        tonemapping::Tonemapping,
    },
    prelude::*,
    window::WindowResolution,
};
use rand::Rng;

const BOUNDS: Vec2 = Vec2::new(1200.0, 800.0);
const PLAYER_SPEED: f32 = 500.0;
const BULLET_SPEED: f32 = 800.0;
const ENEMY_SPEED: f32 = 100.0;
const ENEMY_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const PLAYER_SIZE: Vec2 = Vec2::new(50.0, 30.0);

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
    Victory,
}

#[derive(Resource, Default, PartialEq, Clone, Copy)]
enum GraphicsMode {
    #[default]
    Lightweight,
    Intensive,
}

#[derive(Resource)]
struct GameAssets {
    player: Handle<Image>,
    enemy: Handle<Image>,
}

const ALIEN_ART: &[&str] = &[
    "  X     X  ",
    "   X   X   ",
    "  XXXXXXX  ",
    " XX XXX XX ",
    "XXXXXXXXXXX",
    "X XXXXXXX X",
    "X X     X X",
    "   XX XX   ",
];

const PLAYER_ART: &[&str] = &[
    "     X     ",
    "    XXX    ",
    "    XXX    ",
    " XXXXXXXXX ",
    "XXXXXXXXXXX",
    "XXXXXXXXXXX",
    "XXXXXXXXXXX",
    "XXXXXXXXXXX",
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Invaders WebGPU".to_string(),
                    resolution: WindowResolution::new(BOUNDS.x, BOUNDS.y),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
        )
        .init_state::<AppState>()
        .init_resource::<GraphicsMode>()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)))
        .add_systems(Startup, (setup_camera, setup_assets))
        
        // Main Menu
        .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
        .add_systems(Update, menu_interaction.run_if(in_state(AppState::MainMenu)))
        .add_systems(OnExit(AppState::MainMenu), cleanup::<MainMenuEntity>)
        
        // In Game
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(Update, (
            player_movement,
            player_shooting,
            enemy_movement,
            enemy_shooting,
            bullet_movement,
            collision_system,
        ).run_if(in_state(AppState::InGame)))
        .add_systems(OnExit(AppState::InGame), cleanup::<GameEntity>)
        
        // Game Over
        .add_systems(OnEnter(AppState::GameOver), setup_game_over)
        .add_systems(Update, game_over_input.run_if(in_state(AppState::GameOver)))
        .add_systems(OnExit(AppState::GameOver), cleanup::<GameOverEntity>)
        
        // Victory
        .add_systems(OnEnter(AppState::Victory), setup_victory)
        .add_systems(Update, game_over_input.run_if(in_state(AppState::Victory)))
        .add_systems(OnExit(AppState::Victory), cleanup::<VictoryEntity>)
        .run();
}

// ---- Components ----
#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;
#[derive(Component, PartialEq)] enum BulletType { Player, Enemy }
#[derive(Component)] struct Velocity(Vec2);
#[derive(Component)] struct Collider { size: Vec2 }

// Cleanup Tags
#[derive(Component)] struct MainMenuEntity;
#[derive(Component)] struct GameEntity;
#[derive(Component)] struct GameOverEntity;
#[derive(Component)] struct VictoryEntity;
#[derive(Component)] struct Star;

// UI Components
#[derive(Component)] struct PlayButton;
#[derive(Component)] struct ModeButton;

// ---- Resources ----
#[derive(Resource)] struct EnemyDirection(f32);
#[derive(Resource)] struct EnemyMoveTimer(Timer);
#[derive(Resource)] struct EnemyCount(u32);

// ---- Systems ----

fn setup_assets(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let player_image = create_image_from_ascii(PLAYER_ART);
    let enemy_image = create_image_from_ascii(ALIEN_ART);

    commands.insert_resource(GameAssets {
        player: images.add(player_image),
        enemy: images.add(enemy_image),
    });
}

fn create_image_from_ascii(art: &[&str]) -> Image {
    let width = art[0].len() as u32;
    let height = art.len() as u32;
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for row in art {
        for ch in row.chars() {
            if ch == 'X' {
                data.extend_from_slice(&[255, 255, 255, 255]); // White
            } else {
                data.extend_from_slice(&[0, 0, 0, 0]); // Transparent
            }
        }
    }

    Image::new(
        bevy::render::render_resource::Extent3d { width, height, depth_or_array_layers: 1 },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    )
}

fn setup_camera(mut commands: Commands) {
    // HDR Camera with Bloom for neon glow
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        // We start with Bloom off (Lightweight mode)
        BloomSettings {
            intensity: 0.0,
            ..BloomSettings::NATURAL
        },
    ));
}

fn setup_main_menu(mut commands: Commands, graphics_mode: Res<GraphicsMode>) {
    let mode_text = match *graphics_mode {
        GraphicsMode::Lightweight => "Mode: Lightweight",
        GraphicsMode::Intensive => "Mode: Intensive",
    };

    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        MainMenuEntity,
    )).with_children(|parent| {
        // Title
        parent.spawn(TextBundle::from_section(
            "SPACE INVADERS",
            TextStyle {
                font_size: 80.0,
                color: Color::srgb(0.0, 1.0, 1.0),
                ..default()
            },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(40.0)), ..default() }));

        // Instructions
        parent.spawn(TextBundle::from_section(
            "Controls: Left/Right Arrows or A/D to move.\nSpacebar to shoot.",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(60.0)), ..default() }).with_text_justify(JustifyText::Center));

        // Play Button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(200.0),
                    height: Val::Px(65.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            PlayButton,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "PLAY",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));
        });

        // Mode Toggle Button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                ..default()
            },
            ModeButton,
        )).with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                mode_text,
                TextStyle {
                    font_size: 30.0,
                    color: Color::srgb(0.8, 0.8, 0.8),
                    ..default()
                },
            ));
        });
    });
}

#[allow(clippy::type_complexity)]
fn menu_interaction(
    mut next_state: ResMut<NextState<AppState>>,
    mut graphics_mode: ResMut<GraphicsMode>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&PlayButton>, Option<&ModeButton>, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut camera_query: Query<&mut BloomSettings>,
    mut commands: Commands,
    star_query: Query<Entity, With<Star>>,
) {
    for (interaction, mut color, play_btn, mode_btn, children) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.35, 0.75, 0.35).into();
                
                if play_btn.is_some() {
                    next_state.set(AppState::InGame);
                } else if mode_btn.is_some() {
                    // Toggle Mode
                    let (new_mode, mode_str) = match *graphics_mode {
                        GraphicsMode::Lightweight => (GraphicsMode::Intensive, "Mode: Intensive"),
                        GraphicsMode::Intensive => (GraphicsMode::Lightweight, "Mode: Lightweight"),
                    };
                    *graphics_mode = new_mode;
                    
                    // Update Text
                    if let Ok(mut text) = text_query.get_mut(children[0]) {
                        text.sections[0].value = mode_str.to_string();
                    }

                    // Apply Bloom settings
                    if let Ok(mut bloom) = camera_query.get_single_mut() {
                        bloom.intensity = match *graphics_mode {
                            GraphicsMode::Lightweight => 0.0,
                            GraphicsMode::Intensive => 0.3,
                        };
                    }

                    // Manage Stars
                    match *graphics_mode {
                        GraphicsMode::Lightweight => {
                            for entity in star_query.iter() {
                                commands.entity(entity).despawn();
                            }
                        }
                        GraphicsMode::Intensive => {
                            let mut rng = rand::thread_rng();
                            for _ in 0..150 {
                                let x = rng.gen_range(-BOUNDS.x / 2.0..BOUNDS.x / 2.0);
                                let y = rng.gen_range(-BOUNDS.y / 2.0..BOUNDS.y / 2.0);
                                let size = rng.gen_range(1.0..3.0);
                                let brightness = rng.gen_range(0.2..0.8);
                                
                                commands.spawn((
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::srgba(0.8, 0.9, 1.0, brightness),
                                            custom_size: Some(Vec2::new(size, size)),
                                            ..default()
                                        },
                                        transform: Transform::from_xyz(x, y, -10.0),
                                        ..default()
                                    },
                                    Star,
                                ));
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}

fn setup_game(mut commands: Commands, graphics_mode: Res<GraphicsMode>, assets: Res<GameAssets>) {
    // Determine color intensity based on mode
    let color_multiplier = match *graphics_mode {
        GraphicsMode::Lightweight => 1.0,
        GraphicsMode::Intensive => 3.0,
    };

    // Spawn Player
    commands.spawn((
        SpriteBundle {
            texture: assets.player.clone(),
            sprite: Sprite {
                color: Color::srgb(0.0, 1.0 * color_multiplier, 1.5 * color_multiplier),
                custom_size: Some(PLAYER_SIZE),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -BOUNDS.y / 2.0 + 50.0, 0.0),
            ..default()
        },
        Player,
        Velocity(Vec2::ZERO),
        Collider { size: PLAYER_SIZE },
        GameEntity,
    ));

    // Spawn Enemies
    let rows = 5;
    let cols = 11;
    let spacing = 60.0;
    let start_x = -(cols as f32 * spacing) / 2.0 + spacing / 2.0;
    let start_y = BOUNDS.y / 2.0 - 100.0;

    let enemy_color = Color::srgb(1.5 * color_multiplier, 0.0, 0.6 * color_multiplier);

    for row in 0..rows {
        for col in 0..cols {
            let x = start_x + col as f32 * spacing;
            let y = start_y - row as f32 * spacing;
            
            commands.spawn((
                SpriteBundle {
                    texture: assets.enemy.clone(),
                    sprite: Sprite {
                        color: enemy_color,
                        custom_size: Some(ENEMY_SIZE),
                        ..default()
                    },
                    transform: Transform::from_xyz(x, y, 0.0),
                    ..default()
                },
                Enemy,
                Collider { size: ENEMY_SIZE },
                GameEntity,
            ));
        }
    }

    commands.insert_resource(EnemyDirection(1.0));
    commands.insert_resource(EnemyMoveTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
    commands.insert_resource(EnemyCount((rows * cols) as u32));
}

fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
        let mut direction = 0.0;
        if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
            direction -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
            direction += 1.0;
        }

        velocity.0.x = direction * PLAYER_SPEED;
        transform.translation.x += velocity.0.x * time.delta_seconds();
        
        let half_width = BOUNDS.x / 2.0 - PLAYER_SIZE.x / 2.0;
        transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
    }
}

fn player_shooting(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    graphics_mode: Res<GraphicsMode>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Ok(player_transform) = player_query.get_single() {
            let color_multiplier = match *graphics_mode {
                GraphicsMode::Lightweight => 1.0,
                GraphicsMode::Intensive => 5.0,
            };

            let bullet_size = Vec2::new(4.0, 15.0);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(0.0, 1.0 * color_multiplier, 1.0 * color_multiplier),
                        custom_size: Some(bullet_size),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        player_transform.translation.x,
                        player_transform.translation.y + 20.0,
                        0.0,
                    ),
                    ..default()
                },
                BulletType::Player,
                Velocity(Vec2::new(0.0, BULLET_SPEED)),
                Collider { size: bullet_size },
                GameEntity,
            ));
        }
    }
}

fn enemy_movement(
    mut enemy_query: Query<&mut Transform, With<Enemy>>,
    mut direction: ResMut<EnemyDirection>,
    time: Res<Time>,
) {
    let mut move_down = false;
    let speed = ENEMY_SPEED * time.delta_seconds();
    
    for transform in enemy_query.iter() {
        if transform.translation.x > BOUNDS.x / 2.0 - ENEMY_SIZE.x && direction.0 > 0.0 {
            direction.0 = -1.0;
            move_down = true;
            break;
        } else if transform.translation.x < -BOUNDS.x / 2.0 + ENEMY_SIZE.x && direction.0 < 0.0 {
            direction.0 = 1.0;
            move_down = true;
            break;
        }
    }

    for mut transform in enemy_query.iter_mut() {
        transform.translation.x += direction.0 * speed;
        if move_down {
            transform.translation.y -= 20.0;
        }
    }
}

fn enemy_shooting(
    enemy_query: Query<&Transform, With<Enemy>>,
    graphics_mode: Res<GraphicsMode>,
    mut commands: Commands,
) {
    let mut rng = rand::thread_rng();
    
    let color_multiplier = match *graphics_mode {
        GraphicsMode::Lightweight => 1.0,
        GraphicsMode::Intensive => 5.0,
    };

    for transform in enemy_query.iter() {
        if rng.gen_bool(0.0005) {
            let bullet_size = Vec2::new(6.0, 15.0);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgb(1.0 * color_multiplier, 0.2 * color_multiplier, 0.0),
                        custom_size: Some(bullet_size),
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y - 20.0,
                        0.0,
                    ),
                    ..default()
                },
                BulletType::Enemy,
                Velocity(Vec2::new(0.0, -BULLET_SPEED * 0.5)),
                Collider { size: bullet_size },
                GameEntity,
            ));
        }
    }
}

fn bullet_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<BulletType>>,
    time: Res<Time>,
) {
    for (entity, mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * time.delta_seconds();
        
        if transform.translation.y > BOUNDS.y / 2.0 || transform.translation.y < -BOUNDS.y / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn collision_system(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &Collider, &BulletType)>,
    enemy_query: Query<(Entity, &Transform, &Collider), With<Enemy>>,
    player_query: Query<(Entity, &Transform, &Collider), With<Player>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut enemy_count: ResMut<EnemyCount>,
) {
    for (bullet_entity, bullet_transform, bullet_collider, bullet_type) in bullet_query.iter() {
        let b_pos = bullet_transform.translation.truncate();
        let b_size = bullet_collider.size;

        match bullet_type {
            BulletType::Player => {
                for (enemy_entity, enemy_transform, enemy_collider) in enemy_query.iter() {
                    let e_pos = enemy_transform.translation.truncate();
                    let e_size = enemy_collider.size;

                    if collide(b_pos, b_size, e_pos, e_size) {
                        commands.entity(enemy_entity).despawn();
                        commands.entity(bullet_entity).despawn();
                        enemy_count.0 -= 1;
                        if enemy_count.0 == 0 {
                            next_state.set(AppState::Victory);
                        }
                        break;
                    }
                }
            }
            BulletType::Enemy => {
                if let Ok((player_entity, player_transform, player_collider)) = player_query.get_single() {
                    let p_pos = player_transform.translation.truncate();
                    let p_size = player_collider.size;

                    if collide(b_pos, b_size, p_pos, p_size) {
                        commands.entity(player_entity).despawn();
                        commands.entity(bullet_entity).despawn();
                        next_state.set(AppState::GameOver);
                        break;
                    }
                }
            }
        }
    }
}

fn setup_game_over(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        GameOverEntity,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "GAME OVER",
            TextStyle {
                font_size: 100.0,
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));

        parent.spawn(TextBundle::from_section(
            "Press SPACE to return to Menu",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn setup_victory(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
        VictoryEntity,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "YOU WIN!",
            TextStyle {
                font_size: 100.0,
                color: Color::srgb(0.0, 1.0, 0.0),
                ..default()
            },
        ).with_style(Style { margin: UiRect::bottom(Val::Px(20.0)), ..default() }));

        parent.spawn(TextBundle::from_section(
            "Press SPACE to return to Menu",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        ));
    });
}

fn game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(AppState::MainMenu);
    }
}

// Generic cleanup system for state transitions
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn collide(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let a_min = a_pos - a_size / 2.0;
    let a_max = a_pos + a_size / 2.0;
    let b_min = b_pos - b_size / 2.0;
    let b_max = b_pos + b_size / 2.0;

    a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y
}
