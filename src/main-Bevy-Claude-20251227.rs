use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CONSTANTES DU JEU (les nombres qui ne changent jamais)
// ═══════════════════════════════════════════════════════════════════════════

const PLAYER_SPEED: f32 = 400.0;      // Vitesse du vaisseau (pixels par seconde)
const BULLET_SPEED: f32 = 800.0;      // Vitesse des balles (très rapide !)
const ENEMY_SPEED: f32 = 100.0;       // Vitesse des ennemis (moins rapide)
const MAX_ENEMIES: usize = 10;        // Maximum d'ennemis en même temps
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);   // Taille du vaisseau
const ENEMY_SIZE: Vec2 = Vec2::new(20.0, 20.0);    // Taille des ennemis
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);    // Taille des balles
const PLAYER_HEALTH: i32 = 3;         // Points de vie du joueur
const EXPLOSION_DURATION: f32 = 0.3;  // Durée des explosions en secondes

// ═══════════════════════════════════════════════════════════════════════════
// 📦 COMPOSANTS (les "étiquettes" qu'on colle sur les entités)
// ═══════════════════════════════════════════════════════════════════════════

/// 👾 Étiquette pour le joueur (il n'y en a qu'un)
#[derive(Component)]
struct Player;

/// 💥 Étiquette pour les balles
#[derive(Component)]
struct Bullet;

/// 👽 Étiquette pour les ennemis
#[derive(Component)]
struct Enemy;

/// 🏃 Composant pour tout ce qui peut bouger
#[derive(Component)]
struct Movable {
    velocity: Vec2,
}

/// ❤️ Composant pour la santé (points de vie)
#[derive(Component)]
struct Health {
    current: i32,
}

/// 💥 Composant pour les explosions temporaires
#[derive(Component)]
struct Explosion {
    timer: Timer,  // Durée de vie de l'explosion
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗃️ RESSOURCES (les données partagées par tout le jeu)
// ═══════════════════════════════════════════════════════════════════════════

/// ⏰ Chronomètre pour faire apparaître les ennemis
#[derive(Resource)]
struct EnemySpawnTimer(Timer);

/// 🎯 État du jeu (score, game over, etc.)
#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 SYSTÈME DE DÉMARRAGE (s'exécute UNE SEULE FOIS au début)
// ═══════════════════════════════════════════════════════════════════════════

/// 🏗️ Configure le jeu au démarrage
fn setup_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 📷 Créer une caméra pour voir le jeu
    commands.spawn(Camera2d);
    
    // 🚀 Créer le vaisseau du joueur
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

    // ⏰ Créer le chronomètre pour faire apparaître les ennemis
    commands.insert_resource(
        EnemySpawnTimer(Timer::from_seconds(1.5, TimerMode::Repeating))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 SYSTÈMES D'ENTRÉE (ce qui réagit aux touches du clavier)
// ═══════════════════════════════════════════════════════════════════════════

/// ⌨️ Contrôle le vaisseau avec les flèches ou WASD
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

/// 🔫 Faire tirer le joueur avec la barre ESPACE
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
// 👾 SYSTÈMES D'APPARITION DES ENNEMIS
// ═══════════════════════════════════════════════════════════════════════════

/// 🎲 Fait apparaître des ennemis régulièrement en haut de l'écran
fn enemy_spawner(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    enemy_query: Query<&Enemy>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    
    timer.0.tick(time.delta());

    if timer.0.just_finished() && enemy_query.iter().count() < MAX_ENEMIES {
        let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
        let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
        let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x;
        
        commands.spawn((
            Enemy,
            Movable { 
                velocity: Vec2::new(0.0, -ENEMY_SPEED)
            },
            Sprite {
                image: asset_server.load("sprites/enemy_01.png"),
                custom_size: Some(ENEMY_SIZE),
                ..default()
            },
            Transform::from_xyz(
                x_pos,
                window.height() / 2.0 + ENEMY_SIZE.y,
                0.0
            ),
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈME DE CRÉATION D'EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════

/// 🎬 Crée une explosion visuelle à une position donnée
fn spawn_explosion(
    commands: &mut Commands,
    _asset_server: &Res<AssetServer>,
    position: Vec3,
    size: Vec2,
) {
    commands.spawn((
        Explosion {
            timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
        },
        Sprite {
            // 🎨 OPTION 1 : Utilise une image (si tu as explosion_01.png)
            // image: asset_server.load("sprites/explosion_01.png"),
            
            // 🎨 OPTION 2 : Utilise une couleur (solution temporaire)
            color: Color::srgb(1.0, 0.4, 0.0),  // Orange vif
            custom_size: Some(size),
            ..default()
        },
        Transform::from_translation(position),
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 🏃 SYSTÈME DE MOUVEMENT (fait bouger TOUT ce qui a une velocity)
// ═══════════════════════════════════════════════════════════════════════════

/// 🎯 Applique le mouvement à TOUTES les entités qui ont une velocity
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
// 🚧 SYSTÈME DE CONTRAINTES (garde le joueur dans l'écran)
// ═══════════════════════════════════════════════════════════════════════════

/// 📏 Empêche le joueur de sortir de l'écran
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
// 🗑️ SYSTÈME DE NETTOYAGE (supprime ce qui sort de l'écran)
// ═══════════════════════════════════════════════════════════════════════════

/// 🧹 Supprime les balles et ennemis qui sortent de l'écran
fn despawn_out_of_bounds(
    mut commands: Commands,
    query: Query<(Entity, &Transform, Option<&Bullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    let top_edge = window.height() / 2.0 + 50.0;
    let bottom_edge = -window.height() / 2.0 - 50.0;

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

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈMES DE COLLISION (avec explosions !)
// ═══════════════════════════════════════════════════════════════════════════

/// 🎯 Détecte les collisions entre balles et ennemis
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
                // 💥 NOUVEAU : Créer une explosion à la position de l'ennemi
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,  // Un peu plus grand que l'ennemi
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

/// 💔 Détecte les collisions entre le joueur et les ennemis
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
                // 💥 NOUVEAU : Explosion à la position de l'ennemi
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
                    // 💥 NOUVEAU : Grande explosion pour le joueur
                    spawn_explosion(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,
                        PLAYER_SIZE * 2.0,  // Explosion plus grande !
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

/// ⏰ Supprime les explosions après leur durée de vie
fn cleanup_explosions(
    mut commands: Commands,
    mut explosion_query: Query<(Entity, &mut Explosion)>,
    time: Res<Time>,
) {
    for (entity, mut explosion) in explosion_query.iter_mut() {
        // Faire avancer le chronomètre de l'explosion
        explosion.timer.tick(time.delta());
        
        // Si le temps est écoulé, supprimer l'explosion
        if explosion.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🖥️ SYSTÈME D'AFFICHAGE
// ═══════════════════════════════════════════════════════════════════════════

/// 📊 Affiche les infos dans le terminal
fn display_info(
    _game_state: Res<GameState>,
    player_query: Query<&Health, With<Player>>,
) {
    if let Some(health) = player_query.iter().next() {
        let _ = health;
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
        .add_systems(Startup, setup_game)
        
        // Groupe 1 : INPUT (peuvent tourner en parallèle)
        .add_systems(Update, (
            player_input,
            player_shooting,
            enemy_spawner,
        ))
        
        // Groupe 2 : PHYSICS (doivent être dans l'ordre)
        .add_systems(Update, (
            apply_movement,
            clamp_player_position,
        ).chain())
        
        // Groupe 3 : CLEANUP & COLLISIONS (peuvent tourner en parallèle)
        .add_systems(Update, (
            despawn_out_of_bounds,
            bullet_enemy_collision,
            player_enemy_collision,
            cleanup_explosions,      // 🆕 NOUVEAU SYSTÈME !
            display_info,
        ))
        
        .run();
}

// ═══════════════════════════════════════════════════════════════════════════
// 📝 NOTES SUR LES EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════
//
// 🎬 COMMENT ÇA MARCHE :
// 
// 1. Quand une collision se produit (balle-ennemi ou joueur-ennemi) :
//    → On appelle spawn_explosion() AVANT de supprimer l'entité
//    → L'explosion apparaît exactement à la position de l'objet détruit
//
// 2. L'explosion est une entité temporaire avec :
//    → Un composant Explosion qui contient un Timer
//    → Un Sprite pour l'affichage visuel
//    → Une Transform pour la position
//
// 3. Le système cleanup_explosions() :
//    → Fait avancer le Timer de chaque explosion
//    → Supprime l'explosion quand le temps est écoulé (0.3 secondes)
//
// 🎨 ASTUCE POUR L'IMAGE D'EXPLOSION :
// 
// Si tu n'as pas d'image "explosion_01.png", tu peux :
// - Utiliser temporairement "enemy_01.png" avec une couleur rouge
// - Créer une simple image rouge/orange dans un éditeur
// - Télécharger un sprite d'explosion gratuit (ex: kenney.nl)
// - Ou modifier le code pour utiliser une forme géométrique colorée
//
// Pour utiliser une couleur au lieu d'une image :
// Dans spawn_explosion(), remplace le Sprite par :
//     Sprite {
//         color: Color::srgb(1.0, 0.3, 0.0),  // Orange vif
//         custom_size: Some(size),
//         ..default()
//     },
//
// ═══════════════════════════════════════════════════════════════════════════