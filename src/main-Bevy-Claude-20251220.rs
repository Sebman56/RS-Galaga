// Version de Galaga de ClauddeAI du 2025-12-20

use bevy::{
    prelude::*, 
    window::PrimaryWindow,
};

// --- Constantes de Jeu ---
const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 800.0; 
const ENEMY_SPEED: f32 = 100.0;
const MAX_ENEMIES: usize = 10;
const PLAYER_SIZE: Vec2 = Vec2::new(60.0, 30.0);
const ENEMY_SIZE: Vec2 = Vec2::new(40.0, 40.0);
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);

// --- 1. Composants ---

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Movable {
    velocity: Vec2,
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

// --- 2. Initialisation et Setup (Système `Startup`) ---

fn setup_game(mut commands: Commands) {
    // Bevy 0.17.3: Camera2d suffit (plus besoin de Camera2dBundle)
    commands.spawn(Camera2d);
    
    // Spawn du Joueur
    commands.spawn((
        Player,
        Movable { velocity: Vec2::ZERO },
        Sprite {
            color: Color::srgb(0.0, 0.7, 1.0), // Bleu
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0), // Bas de l'écran
    ));

    commands.insert_resource(EnemySpawnTimer(Timer::from_seconds(1.5, TimerMode::Repeating)));
}

// --- 3. Systèmes de Jeu (Systèmes `Update`) ---

// Gère l'apparition régulière des ennemis
fn enemy_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    enemy_query: Query<&Enemy>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    timer.0.tick(time.delta());

    if timer.0.just_finished() && enemy_query.iter().count() < MAX_ENEMIES {
        let window = window_query.single().unwrap();
        let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
        
        let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x; 
        
        commands.spawn((
            Enemy,
            Movable { velocity: Vec2::new(0.0, -ENEMY_SPEED) }, // Descend
            Sprite {
                color: Color::srgb(1.0, 0.3, 0.3), // Rouge
                custom_size: Some(ENEMY_SIZE),
                ..default()
            },
            Transform::from_xyz(x_pos, window.height() / 2.0 + ENEMY_SIZE.y, 0.0),
        ));
    }
}

// Gère le mouvement du joueur
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Movable), With<Player>>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let mut direction = 0.0;
    
    if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }
    
    let window = window_query.single().unwrap();
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;

    for (mut transform, mut movable) in query.iter_mut() {
        movable.velocity.x = direction * PLAYER_SPEED;

        let delta_x = movable.velocity.x * time.delta_secs();
        transform.translation.x += delta_x;

        // Limite le joueur à l'écran
        transform.translation.x = transform.translation.x.clamp(-limit_x, limit_x);
    }
}

// Gère le tir du joueur
fn player_shooting(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for player_transform in query.iter() {
            let spawn_pos = player_transform.translation + Vec3::new(0.0, PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0, 0.0);
            
            commands.spawn((
                Bullet,
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite {
                    color: Color::srgb(1.0, 1.0, 0.0),
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                Transform::from_translation(spawn_pos),
            ));
        }
    }
}

// Applique le mouvement général (utilisé par les ennemis et les balles)
fn apply_movement(mut query: Query<(&Movable, &mut Transform)>, time: Res<Time>) {
    for (movable, mut transform) in query.iter_mut() {
        transform.translation += (movable.velocity * time.delta_secs()).extend(0.0);
    }
}

// Supprime les entités qui sortent de l'écran 
fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&Bullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().unwrap();
    let top_edge = window.height() / 2.0 + 50.0; // Marge
    let bottom_edge = -window.height() / 2.0 - 50.0; // Marge

    for (entity, transform, is_bullet, is_enemy) in query.iter() {
        let y = transform.translation.y;
        
        if is_bullet.is_some() && y > top_edge {
            commands.entity(entity).despawn();
        }
        
        if is_enemy.is_some() && y < bottom_edge {
            commands.entity(entity).despawn();
        }
    }
}

// Détection de collision Bullet-Enemy
fn collision_detection(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        let bullet_center = bullet_transform.translation.xy();
        let bullet_half_size = BULLET_SIZE / 2.0;

        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_center = enemy_transform.translation.xy();
            let enemy_half_size = ENEMY_SIZE / 2.0;

            let collision = (bullet_center.x - enemy_center.x).abs() < (bullet_half_size.x + enemy_half_size.x)
                && (bullet_center.y - enemy_center.y).abs() < (bullet_half_size.y + enemy_half_size.y);

            if collision {
                commands.entity(enemy_entity).despawn();
                commands.entity(bullet_entity).despawn(); 
                break; 
            }
        }
    }
}


// --- 4. Fonction Principale (main) ---

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08))) 
        .add_systems(Startup, setup_game)
        .add_systems(
            Update, 
            (
                player_movement,
                player_shooting,
                enemy_spawner,
                apply_movement,
                despawn_out_of_bounds,
                collision_detection,
            )
        )
        .run();
}