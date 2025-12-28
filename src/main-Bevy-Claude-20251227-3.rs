// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Claude  AI le 2025-12-27
// ═══════════════════════════════════════════════════════════════════════════



use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CONSTANTES DU JEU
// ═══════════════════════════════════════════════════════════════════════════

const PLAYER_SPEED: f32 = 400.0;
const BULLET_SPEED: f32 = 800.0;
const ENEMY_SPEED: f32 = 100.0;
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);
const ENEMY_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);
const PLAYER_HEALTH: i32 = 3;
const EXPLOSION_DURATION: f32 = 0.3;

// 🆕 CONSTANTES POUR LES VAGUES
const ENEMIES_PER_WAVE: usize = 10;        // Nombre d'ennemis par vague
const TIME_BETWEEN_SPAWNS: f32 = 0.5;      // Temps entre chaque ennemi (secondes)
const TIME_BETWEEN_WAVES: f32 = 5.0;       // Temps entre chaque vague (secondes)

// ═══════════════════════════════════════════════════════════════════════════
// 📦 COMPOSANTS
// ═══════════════════════════════════════════════════════════════════════════

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

#[derive(Component)]
struct Health {
    current: i32,
}

#[derive(Component)]
struct Explosion {
    timer: Timer,
}

// ═══════════════════════════════════════════════════════════════════════════
// 🆕 ÉNUMÉRATIONS POUR LES VAGUES
// ═══════════════════════════════════════════════════════════════════════════

/// 🎯 Direction d'où viennent les ennemis
#[derive(Clone, Copy, Debug, PartialEq)]
enum SpawnDirection {
    Top,        // Haut de l'écran
    Left,       // Côté gauche
    Right,      // Côté droit
}

/// 🌊 État d'une vague d'ennemis
#[derive(Clone, Copy, Debug, PartialEq)]
enum WaveState {
    Spawning,   // En train de faire apparaître les ennemis
    Fighting,   // Les ennemis sont là, on attend qu'ils soient tous morts
    Waiting,    // Pause avant la prochaine vague
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗃️ RESSOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// 🌊 Gestionnaire de vagues d'ennemis
#[derive(Resource)]
struct WaveManager {
    current_wave: u32,              // Numéro de la vague actuelle
    state: WaveState,               // État actuel
    direction: SpawnDirection,      // Direction de la vague actuelle
    enemies_spawned: usize,         // Nombre d'ennemis déjà apparus
    spawn_timer: Timer,             // Timer entre chaque ennemi
    wave_timer: Timer,              // Timer entre les vagues
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
    /// 🔄 Passe à la vague suivante
    fn next_wave(&mut self) {
        self.current_wave += 1;
        self.enemies_spawned = 0;
        self.state = WaveState::Spawning;
        
        // 🎲 Choisir la direction selon le numéro de vague
        // Vague 1, 4, 7... = Haut
        // Vague 2, 5, 8... = Gauche
        // Vague 3, 6, 9... = Droite
        self.direction = match self.current_wave % 3 {
            1 => SpawnDirection::Top,
            2 => SpawnDirection::Left,
            _ => SpawnDirection::Right,
        };
        
        println!("🌊 VAGUE {} - Direction: {:?}", self.current_wave, self.direction);
    }
}

#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 SYSTÈME DE DÉMARRAGE
// ═══════════════════════════════════════════════════════════════════════════

fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
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
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 SYSTÈMES D'ENTRÉE
// ═══════════════════════════════════════════════════════════════════════════

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Movable, With<Player>>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    
    let mut direction = 0.0;
    
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }
    
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }
    
    for mut movable in player_query.iter_mut() {
        movable.velocity.x = direction * PLAYER_SPEED;
    }
}

fn player_shooting(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    
    if keyboard.just_pressed(KeyCode::Space) {
        for player_transform in player_query.iter() {
            let spawn_pos = player_transform.translation + Vec3::new(
                0.0,
                PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,
                0.0
            );
            
            commands.spawn((
                Bullet,
                Movable { 
                    velocity: Vec2::new(0.0, BULLET_SPEED)
                },
                Sprite {
                    image: asset_server.load("sprites/bullet_01.png"),
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                Transform::from_translation(spawn_pos),
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🆕 SYSTÈME DE GESTION DES VAGUES
// ═══════════════════════════════════════════════════════════════════════════

/// 🌊 Gère l'apparition des vagues d'ennemis
fn wave_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut wave_manager: ResMut<WaveManager>,
    enemy_query: Query<&Enemy>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    let enemy_count = enemy_query.iter().count();
    
    match wave_manager.state {
        // 📍 État SPAWNING : faire apparaître les ennemis de la vague
        WaveState::Spawning => {
            wave_manager.spawn_timer.tick(time.delta());
            
            if wave_manager.spawn_timer.just_finished() 
                && wave_manager.enemies_spawned < ENEMIES_PER_WAVE {
                
                // Faire apparaître un ennemi selon la direction
                spawn_enemy_from_direction(
                    &mut commands,
                    &asset_server,
                    window,
                    wave_manager.direction,
                    wave_manager.enemies_spawned,
                );
                
                wave_manager.enemies_spawned += 1;
                
                // Si tous les ennemis sont apparus, passer à l'état Fighting
                if wave_manager.enemies_spawned >= ENEMIES_PER_WAVE {
                    wave_manager.state = WaveState::Fighting;
                    println!("⚔️ Tous les ennemis sont là ! Combattez !");
                }
            }
        }
        
        // ⚔️ État FIGHTING : attendre que tous les ennemis soient morts
        WaveState::Fighting => {
            if enemy_count == 0 {
                wave_manager.state = WaveState::Waiting;
                wave_manager.wave_timer.reset();
                println!("✅ Vague {} terminée ! Prochaine vague dans {}s...", 
                    wave_manager.current_wave, TIME_BETWEEN_WAVES);
            }
        }
        
        // ⏰ État WAITING : pause avant la prochaine vague
        WaveState::Waiting => {
            wave_manager.wave_timer.tick(time.delta());
            
            if wave_manager.wave_timer.finished() {
                wave_manager.next_wave();
            }
        }
    }
}

/// 🎯 Fait apparaître un ennemi depuis une direction donnée
fn spawn_enemy_from_direction(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    window: &Window,
    direction: SpawnDirection,
    index: usize,
) {
    let (position, velocity) = match direction {
        // ⬇️ Haut : les ennemis descendent
        SpawnDirection::Top => {
            let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
            let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x;
            let y_pos = window.height() / 2.0 + ENEMY_SIZE.y;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(0.0, -ENEMY_SPEED),
            )
        }
        
        // ➡️ Gauche : les ennemis vont vers la droite
        SpawnDirection::Left => {
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            // Espacer les ennemis verticalement
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            let x_pos = -window.width() / 2.0 - ENEMY_SIZE.x;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(ENEMY_SPEED, 0.0),
            )
        }
        
        // ⬅️ Droite : les ennemis vont vers la gauche
        SpawnDirection::Right => {
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            let x_pos = window.width() / 2.0 + ENEMY_SIZE.x;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(-ENEMY_SPEED, 0.0),
            )
        }
    };
    
    commands.spawn((
        Enemy,
        Movable { velocity },
        Sprite {
            image: asset_server.load("sprites/enemy_01.png"),
            custom_size: Some(ENEMY_SIZE),
            ..default()
        },
        Transform::from_translation(position),
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈME DE CRÉATION D'EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════

fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: Vec3,
    size: Vec2,
) {
    commands.spawn((
        Explosion {
            timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
        },
        Sprite {
//            color: Color::srgb(1.0, 0.4, 0.0),
            image: asset_server.load("sprites/explosion_01.png"),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position),
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 🏃 SYSTÈME DE MOUVEMENT
// ═══════════════════════════════════════════════════════════════════════════

fn apply_movement(
    mut query: Query<(&Movable, &mut Transform)>,
    time: Res<Time>,
) {
    for (movable, mut transform) in query.iter_mut() {
        let movement = movable.velocity * time.delta_secs();
        transform.translation += movement.extend(0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚧 SYSTÈME DE CONTRAINTES
// ═══════════════════════════════════════════════════════════════════════════

fn clamp_player_position(
    mut player_query: Query<&mut Transform, With<Player>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;
    
    for mut transform in player_query.iter_mut() {
        transform.translation.x = transform.translation.x.clamp(-limit_x, limit_x);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗑️ SYSTÈME DE NETTOYAGE
// ═══════════════════════════════════════════════════════════════════════════

fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&Bullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    
    // 🆕 Marges plus grandes pour les ennemis venant des côtés
    let top_edge = window.height() / 2.0 + 50.0;
    let bottom_edge = -window.height() / 2.0 - 50.0;
    let left_edge = -window.width() / 2.0 - 50.0;
    let right_edge = window.width() / 2.0 + 50.0;

    for (entity, transform, is_bullet, is_enemy) in query.iter() {
        let pos = transform.translation;
        
        // Balles : seulement en haut
        if is_bullet.is_some() && pos.y > top_edge {
            commands.entity(entity).despawn();
        }
        
        // Ennemis : tous les bords
        if is_enemy.is_some() {
            if pos.y < bottom_edge || pos.y > top_edge 
                || pos.x < left_edge || pos.x > right_edge {
                commands.entity(entity).despawn();
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈMES DE COLLISION
// ═══════════════════════════════════════════════════════════════════════════

fn bullet_enemy_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut game_state: ResMut<GameState>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        let bullet_pos = bullet_transform.translation.xy();
        let bullet_half = BULLET_SIZE / 2.0;
        
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();
            let enemy_half = ENEMY_SIZE / 2.0;
            
            let dx = (bullet_pos.x - enemy_pos.x).abs();
            let dy = (bullet_pos.y - enemy_pos.y).abs();
            
            let collision = dx < (bullet_half.x + enemy_half.x) 
                         && dy < (bullet_half.y + enemy_half.y);

            if collision {
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,
                );
                
                commands.entity(enemy_entity).despawn();
                commands.entity(bullet_entity).despawn();
                game_state.score += 10;
                println!("💥 Touché ! Score : {}", game_state.score);
                break;
            }
        }
    }
}

fn player_enemy_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut game_state: ResMut<GameState>,
) {
    if game_state.game_over {
        return;
    }

    for (player_entity, player_transform, mut health) in player_query.iter_mut() {
        let player_pos = player_transform.translation.xy();
        let player_half = PLAYER_SIZE / 2.0;

        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();
            let enemy_half = ENEMY_SIZE / 2.0;

            let dx = (player_pos.x - enemy_pos.x).abs();
            let dy = (player_pos.y - enemy_pos.y).abs();
            
            let collision = dx < (player_half.x + enemy_half.x)
                         && dy < (player_half.y + enemy_half.y);

            if collision {
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,
                );
                
                commands.entity(enemy_entity).despawn();
                health.current -= 1;
                println!("💔 Aïe ! Vies restantes : {}", health.current);
                
                if health.current <= 0 {
                    spawn_explosion(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,
                        PLAYER_SIZE * 2.0,
                    );
                    
                    commands.entity(player_entity).despawn();
                    game_state.game_over = true;
                    println!("☠️ GAME OVER ! Score final : {}", game_state.score);
                }
                break;
            }
        }
        
        if game_state.game_over {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧹 SYSTÈME DE NETTOYAGE DES EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════

fn cleanup_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Explosion)>,
    time: Res<Time>,
) {
    for (entity, mut explosion) in explosion_query.iter_mut() {
        explosion.timer.tick(time.delta());
        
        if explosion.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🖥️ SYSTÈME D'AFFICHAGE
// ═══════════════════════════════════════════════════════════════════════════

fn display_info(
    wave_manager: Res<WaveManager>,
    game_state: Res<GameState>,
    player_query: Query<&Health, With<Player>>,
) {
    if let Some(health) = player_query.iter().next() {
        // Afficher les infos toutes les 120 frames (2 secondes à 60 FPS)
        let _ = (wave_manager.current_wave, game_state.score, health.current);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚀 FONCTION PRINCIPALE
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        .init_resource::<GameState>()
        .init_resource::<WaveManager>()  // 🆕 Nouveau !
        .add_systems(Startup, setup_game)
        
        .add_systems(Update, (
            player_input,
            player_shooting,
            wave_spawner,  // 🆕 Remplace enemy_spawner
        ))
        
        .add_systems(Update, (
            apply_movement,
            clamp_player_position,
        ).chain())
        
        .add_systems(Update, (
            despawn_out_of_bounds,
            bullet_enemy_collision,
            player_enemy_collision,
            cleanup_explosions,
            display_info,
        ))
        
        .run();
}

// ═══════════════════════════════════════════════════════════════════════════
// 📚 RÉCAPITULATIF DU SYSTÈME DE VAGUES
// ═══════════════════════════════════════════════════════════════════════════
//
// 🌊 COMMENT ÇA MARCHE :
//
// 1. État SPAWNING :
//    - Fait apparaître 10 ennemis un par un (0.5s entre chaque)
//    - La direction change selon la vague (haut/gauche/droite)
//    - Passe à Fighting quand les 10 sont apparus
//
// 2. État FIGHTING :
//    - Attend que le joueur tue tous les ennemis
//    - Vérifie si enemy_count == 0
//    - Passe à Waiting quand c'est terminé
//
// 3. État WAITING :
//    - Pause de 5 secondes
//    - Le joueur peut souffler un peu !
//    - Lance la vague suivante après le timer
//
// 🎯 DIRECTIONS :
//    - Vague 1, 4, 7... → Haut (descendent)
//    - Vague 2, 5, 8... → Gauche (vont à droite)
//    - Vague 3, 6, 9... → Droite (vont à gauche)
//
// 📊 DIFFICULTÉS POSSIBLES :
//    - Augmenter ENEMY_SPEED selon wave_manager.current_wave
//    - Augmenter ENEMIES_PER_WAVE tous les 3 niveaux
//    - Ajouter des patterns de mouvement plus complexes
//
// ═══════════════════════════════════════════════════════════════════════════