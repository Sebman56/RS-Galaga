// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-12-28 04h00
// ═══════════════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Claude AI le 2025-12-28
// ═══════════════════════════════════════════════════════════════════════════

// 📦 On importe les outils de Bevy pour faire notre jeu
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::math::Vec3Swizzles; // 👈 Indispensable pour les collisions (.xy())

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CONSTANTES DU JEU
// ═══════════════════════════════════════════════════════════════════════════
const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 800.0;
const ENEMY_SPEED: f32 = 100.0;
const ENEMY_BULLET_SPEED: f32 = 400.0; // 🆕 Vitesse des tirs ennemis
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);
const ENEMY_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);
const PLAYER_HEALTH: i32 = 3;
const EXPLOSION_DURATION: f32 = 0.3;

const ENEMIES_PER_WAVE: usize = 10;
const TIME_BETWEEN_SPAWNS: f32 = 0.5;
const TIME_BETWEEN_WAVES: f32 = 5.0;
const ENEMY_SHOOT_INTERVAL: f32 = 2.0; // 🆕 Un ennemi tire toutes les 2 secondes

// ═══════════════════════════════════════════════════════════════════════════
// 📦 COMPOSANTS
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Component)]
struct Player;

#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct LevelText;
#[derive(Component)]
struct LivesText;

// 🆕 Distinction entre balles du joueur et des ennemis
#[derive(Component)]
struct PlayerBullet; // Balle bleue du joueur

#[derive(Component)]
struct EnemyBullet; // Balle rouge de l'ennemi

// 🆕 Timer pour le tir des ennemis
#[derive(Component)]
struct EnemyShootTimer {
    timer: Timer,
}

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Movable {
    velocity: Vec2,
}

#[derive(Component)]
struct Health {
    current: i32,
}

#[derive(Component)]
struct Explosion {
    timer: Timer,
}

// ═══════════════════════════════════════════════════════════════════════════
// 🌊 SYSTÈME DE VAGUES (ÉNUMÉRATIONS & RESSOURCES)
// ═══════════════════════════════════════════════════════════════════════════
#[derive(Clone, Copy, Debug, PartialEq)]
enum SpawnDirection {
    Top, Left, Right,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum WaveState {
    Spawning, Fighting, Waiting,
}

#[derive(Resource)]
struct WaveManager {
    current_wave: u32,
    state: WaveState,
    direction: SpawnDirection,
    enemies_spawned: usize,
    spawn_timer: Timer,
    wave_timer: Timer,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 1,
            state: WaveState::Spawning,
            direction: SpawnDirection::Top,
            enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(TIME_BETWEEN_SPAWNS, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(TIME_BETWEEN_WAVES, TimerMode::Once),
        }
    }
}

impl WaveManager {
    fn next_wave(&mut self) {
        self.current_wave += 1;
        self.enemies_spawned = 0;
        self.state = WaveState::Spawning;
        self.direction = match self.current_wave % 3 {
            1 => SpawnDirection::Top,
            2 => SpawnDirection::Left,
            _ => SpawnDirection::Right,
        };
    }
}

#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 SYSTÈMES
// ═══════════════════════════════════════════════════════════════════════════

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    
    commands.spawn((
        Player,
        Movable { velocity: Vec2::ZERO },
        Health { current: PLAYER_HEALTH },
        Sprite {
            image: asset_server.load("sprites/player_01.png"),
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));

    // UI (Score, Vagues, Vies)
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(20.0)),
        ..default()
    })
    .with_child((BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),))
    .with_children(|parent| {
        parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 30.0, ..default() }, TextColor(Color::WHITE)));
        parent.spawn((LevelText, Text::new("Vague: 1"), TextFont { font_size: 30.0, ..default() }, TextColor(Color::WHITE)));
        parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 30.0, ..default() }, TextColor(Color::WHITE)));
    });
}

fn player_input(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>, state: Res<GameState>) {
    if state.game_over { return; }
    let mut dir = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) { dir -= 1.0; }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) { dir += 1.0; }
    for mut m in query.iter_mut() { m.velocity.x = dir * PLAYER_SPEED; }
}

// 🔵 Tir du joueur avec laser BLEU
fn player_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
    state: Res<GameState>
) {
    if state.game_over { return; }
    if keyboard.just_pressed(KeyCode::Space) {
        for t in query.iter() {
            let spawn_pos = t.translation + Vec3::new(0.0, PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0, 0.0);
            commands.spawn((
                PlayerBullet, // 🆕 Étiquette "balle du joueur"
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite { 
                    image: asset_server.load("sprites/laser_blue.png"), // 🔵 Laser bleu
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                Transform::from_translation(spawn_pos),
            ));
        }
    }
}

// 🆕 🔴 TIR DES ENNEMIS avec laser ROUGE
fn enemy_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut enemy_query: Query<(&Transform, &mut EnemyShootTimer), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
    state: Res<GameState>,
) {
    if state.game_over { return; }
    
    // Si le joueur existe
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;
        
        // Pour chaque ennemi
        for (enemy_transform, mut shoot_timer) in enemy_query.iter_mut() {
            // Faire avancer le timer
            shoot_timer.timer.tick(time.delta());
            
            // Si le timer a sonné, l'ennemi tire !
            if shoot_timer.timer.just_finished() {
                let enemy_pos = enemy_transform.translation;
                
                // 🎯 Calculer la direction vers le joueur
                let direction = (player_pos - enemy_pos).normalize().xy();
                
                // Position de départ du tir (sous l'ennemi)
                let spawn_pos = enemy_pos + Vec3::new(0.0, -ENEMY_SIZE.y / 2.0 - BULLET_SIZE.y / 2.0, 0.0);
                
                // 🔴 Créer le laser rouge
                commands.spawn((
                    EnemyBullet, // Étiquette "balle ennemie"
                    Movable { velocity: direction * ENEMY_BULLET_SPEED },
                    Sprite {
                        image: asset_server.load("sprites/laser_red.png"), // 🔴 Laser rouge
                        custom_size: Some(BULLET_SIZE),
                        ..default()
                    },
                    Transform::from_translation(spawn_pos),
                ));
            }
        }
    }
}

fn wave_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut wave_manager: ResMut<WaveManager>,
    enemy_query: Query<&Enemy>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    game_state: Res<GameState>,
) {
    if game_state.game_over { return; }
    
    let window = window_query.single().expect("Pas de fenêtre");
    let enemy_count = enemy_query.iter().count();
    
    match wave_manager.state {
        WaveState::Spawning => {
            wave_manager.spawn_timer.tick(time.delta());
            if wave_manager.spawn_timer.just_finished() && wave_manager.enemies_spawned < ENEMIES_PER_WAVE {
                spawn_enemy_from_direction(&mut commands, &asset_server, window, wave_manager.direction, wave_manager.enemies_spawned);
                wave_manager.enemies_spawned += 1;
                if wave_manager.enemies_spawned >= ENEMIES_PER_WAVE { wave_manager.state = WaveState::Fighting; }
            }
        }
        WaveState::Fighting => {
            if enemy_count == 0 {
                wave_manager.state = WaveState::Waiting;
                wave_manager.wave_timer.reset();
            }
        }
        WaveState::Waiting => {
            wave_manager.wave_timer.tick(time.delta());
            if wave_manager.wave_timer.is_finished() { wave_manager.next_wave(); }
        }
    }
}

fn spawn_enemy_from_direction(commands: &mut Commands, asset_server: &Res<AssetServer>, window: &Window, direction: SpawnDirection, index: usize) {
    let (position, velocity) = match direction {
        SpawnDirection::Top => {
            let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
            let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x;
            (Vec3::new(x_pos, window.height() / 2.0 + ENEMY_SIZE.y, 0.0), Vec2::new(0.0, -ENEMY_SPEED))
        }
        SpawnDirection::Left => {
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            (Vec3::new(-window.width() / 2.0 - ENEMY_SIZE.x, y_pos, 0.0), Vec2::new(ENEMY_SPEED, 0.0))
        }
        SpawnDirection::Right => {
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            (Vec3::new(window.width() / 2.0 + ENEMY_SIZE.x, y_pos, 0.0), Vec2::new(-ENEMY_SPEED, 0.0))
        }
    };

    // 🆕 Chaque ennemi a son propre timer de tir aléatoire
    let random_delay = rand::random::<f32>() * ENEMY_SHOOT_INTERVAL;
    
    commands.spawn((
        Enemy,
        Movable { velocity },
        EnemyShootTimer { // 🆕 Timer pour tirer
            timer: Timer::from_seconds(random_delay + ENEMY_SHOOT_INTERVAL, TimerMode::Repeating),
        },
        Sprite { image: asset_server.load("sprites/enemy_01.png"), custom_size: Some(ENEMY_SIZE), ..default() },
        Transform::from_translation(position),
    ));
}

fn spawn_explosion(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec3, size: Vec2) {
    commands.spawn((
        Explosion { timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once) },
        Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(size), ..default() },
        Transform::from_translation(pos),
    ));
}

fn apply_movement(mut query: Query<(&Movable, &mut Transform)>, time: Res<Time>) {
    for (m, mut t) in query.iter_mut() {
        t.translation += (m.velocity * time.delta_secs()).extend(0.0);
    }
}

fn clamp_player_position(mut query: Query<&mut Transform, With<Player>>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().expect("Pas de fenêtre");
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;
    for mut t in query.iter_mut() { t.translation.x = t.translation.x.clamp(-limit_x, limit_x); }
}

// 🆕 Nettoyer TOUTES les balles (bleues ET rouges)
fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&PlayerBullet>, Option<&EnemyBullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.single().expect("Pas de fenêtre");
    let h = window.height() / 2.0 + 50.0;
    let w = window.width() / 2.0 + 50.0;
    
    for (e, t, is_player_bullet, is_enemy_bullet, is_enemy) in query.iter() {
        let p = t.translation;
        
        // Balles du joueur : sortent en haut
        if is_player_bullet.is_some() && p.y > h { 
            commands.entity(e).despawn(); 
        }
        
        // Balles des ennemis : sortent en bas
        if is_enemy_bullet.is_some() && p.y < -h { 
            commands.entity(e).despawn(); 
        }
        
        // Ennemis : tous les bords
        if is_enemy.is_some() && (p.y < -h || p.y > h || p.x < -w || p.x > w) { 
            commands.entity(e).despawn(); 
        }
    }
}

// 🔵 Collision : Balle BLEUE du joueur VS Ennemi
fn player_bullet_enemy_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bullet_query: Query<(Entity, &Transform), With<PlayerBullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut state: ResMut<GameState>
) {
    for (b_e, b_t) in bullet_query.iter() {
        let b_pos = b_t.translation.xy();
        for (e_e, e_t) in enemy_query.iter() {
            let e_pos = e_t.translation.xy();
            if (b_pos.x - e_pos.x).abs() < (BULLET_SIZE.x + ENEMY_SIZE.x) / 2.0 
                && (b_pos.y - e_pos.y).abs() < (BULLET_SIZE.y + ENEMY_SIZE.y) / 2.0 {
                spawn_explosion(&mut commands, &asset_server, e_t.translation, ENEMY_SIZE * 1.5);
                commands.entity(e_e).despawn();
                commands.entity(b_e).despawn();
                state.score += 10;
                break;
            }
        }
    }
}

// 🆕 🔴 Collision : Balle ROUGE de l'ennemi VS Joueur
fn enemy_bullet_player_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bullet_query: Query<(Entity, &Transform), With<EnemyBullet>>,
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>,
    mut state: ResMut<GameState>
) {
    if state.game_over { return; }
    
    for (p_e, p_t, mut h) in player_query.iter_mut() {
        let p_pos = p_t.translation.xy();
        
        for (b_e, b_t) in bullet_query.iter() {
            let b_pos = b_t.translation.xy();
            
            if (p_pos.x - b_pos.x).abs() < (PLAYER_SIZE.x + BULLET_SIZE.x) / 2.0 
                && (p_pos.y - b_pos.y).abs() < (PLAYER_SIZE.y + BULLET_SIZE.y) / 2.0 {
                
                // Détruire la balle
                commands.entity(b_e).despawn();
                
                // Explosion à l'impact
                spawn_explosion(&mut commands, &asset_server, b_t.translation, BULLET_SIZE * 3.0);
                
                // Perdre une vie
                h.current -= 1;
                
                if h.current <= 0 {
                    spawn_explosion(&mut commands, &asset_server, p_t.translation, PLAYER_SIZE * 2.0);
                    commands.entity(p_e).despawn();
                    state.game_over = true;
                }
                break;
            }
        }
        
        if state.game_over { break; }
    }
}

// Collision : Joueur VS Ennemi (contact direct)
fn player_enemy_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut state: ResMut<GameState>
) {
    if state.game_over { return; }
    
    for (p_e, p_t, mut h) in player_query.iter_mut() {
        let p_pos = p_t.translation.xy();
        for (e_e, e_t) in enemy_query.iter() {
            let e_pos = e_t.translation.xy();
            if (p_pos.x - e_pos.x).abs() < (PLAYER_SIZE.x + ENEMY_SIZE.x) / 2.0 
                && (p_pos.y - e_pos.y).abs() < (PLAYER_SIZE.y + ENEMY_SIZE.y) / 2.0 {
                spawn_explosion(&mut commands, &asset_server, e_t.translation, ENEMY_SIZE * 1.5);
                commands.entity(e_e).despawn();
                h.current -= 1;
                if h.current <= 0 {
                    spawn_explosion(&mut commands, &asset_server, p_t.translation, PLAYER_SIZE * 2.0);
                    commands.entity(p_e).despawn();
                    state.game_over = true;
                }
                break;
            }
        }
        if state.game_over { break; }
    }
}

fn cleanup_explosions(mut commands: Commands, mut query: Query<(Entity, &mut Explosion)>, time: Res<Time>) {
    for (e, mut ex) in query.iter_mut() {
        ex.timer.tick(time.delta());
        if ex.timer.is_finished() { commands.entity(e).despawn(); }
    }
}

fn display_info(
    wave: Res<WaveManager>,
    state: Res<GameState>,
    p_query: Query<&Health, With<Player>>,
    mut s_text: Query<&mut Text, With<ScoreText>>,
    mut l_text: Query<&mut Text, (With<LevelText>, Without<ScoreText>, Without<LivesText>)>,
    mut liv_text: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<LevelText>)>
) {
    if let Ok(mut t) = s_text.single_mut() { **t = format!("Score: {}", state.score); }
    if let Ok(mut t) = l_text.single_mut() { **t = format!("Vague: {}", wave.current_wave); }
    if let Ok(mut t) = liv_text.single_mut() {
        if let Some(h) = p_query.iter().next() { 
            **t = format!("Vies: {}", h.current); 
        } else { 
            **t = "Vies: 0".to_string(); 
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        .init_resource::<GameState>()
        .init_resource::<WaveManager>()
        .add_systems(Startup, setup_game)
        .add_systems(Update, (
            player_input,
            player_shooting,
            enemy_shooting,  // 🆕 Système de tir des ennemis
            wave_spawner
        ))
        .add_systems(Update, (apply_movement, clamp_player_position).chain())
        .add_systems(Update, (
            despawn_out_of_bounds,
            player_bullet_enemy_collision,  // 🔵 Balle bleue VS ennemi
            enemy_bullet_player_collision,  // 🆕 🔴 Balle rouge VS joueur
            player_enemy_collision,
            cleanup_explosions,
            display_info
        ))
        .run();
}