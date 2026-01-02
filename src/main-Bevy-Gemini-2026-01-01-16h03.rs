// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 16h03
// ═══════════════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════════════
// 🚀 MON SUPER XGALAGA : LE CODE COMPLET ET EXPLIQUÉ !
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*; // On sort la caisse à outils Bevy pour fabriquer le jeu
use bevy::window::PrimaryWindow; // On prépare l'outil pour parler à la fenêtre
use bevy::app::AppExit; // Le bouton pour quitter proprement
use bevy::math::Vec3Swizzles; // Une astuce pour calculer les positions X et Y

// 🎮 LES RÉGLAGES (Tes chiffres magiques pour régler le jeu)
const PLAYER_SPEED: f32 = 500.0;     // Vitesse de ton vaisseau
const BULLET_SPEED: f32 = 700.0;     // Vitesse de tes lasers
const ENEMY_SPEED: f32 = 120.0;      // Vitesse des méchants aliens
const PLAYER_SIZE: Vec2 = Vec2::new(35.0, 20.0); // Taille du vaisseau
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0);  // Taille d'un alien
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0);   // Taille d'une balle laser
const PLAYER_HEALTH: i32 = 3;        // Tes 3 coeurs au début

// 📦 LES ÉTIQUETTES (On colle des noms sur les objets)
#[derive(Component)] struct Player; // Étiquette "Moi"
#[derive(Component)] struct Bullet { from_player: bool } // Étiquette "Balle"
#[derive(Component)] struct Movable { velocity: Vec2 }   // Étiquette "Je bouge"
#[derive(Component)] struct Health { current: i32 }      // Étiquette "Vie"
#[derive(Component)] struct EnemyFireTimer(Timer);       // Montre pour tirer
#[derive(Component)] struct Explosion { timer: Timer }   // Effet "Boum"
#[derive(Component)] struct FloatingText { timer: Timer } // Score qui vole

#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss } // Soldat ou Chef
#[derive(Component)] struct Enemy { kind: EnemyType } // L'alien et son type

// Étiquettes pour les textes sur l'écran
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct LivesText;
#[derive(Component)] struct MainMessage;

// 🌊 DIRECTIONS ET ÉTATS (D'où ils viennent et ce qu'ils font)
#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right }
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting }

// 🗃️ LE CERVEAU (Il retient tout ce qui se passe)
#[derive(Resource)]
struct WaveManager {
    current_level: u32,
    current_wave: u32,
    state: WaveState,
    direction: SpawnDirection,
    enemies_spawned: usize,
    spawn_timer: Timer,
    wave_timer: Timer,
    enemies_killed: usize, // 🎯 Compte tes trophées
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_level: 1, current_wave: 1, state: WaveState::Spawning,
            direction: SpawnDirection::Top, enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(4.0, TimerMode::Once),
            enemies_killed: 0,
        }
    }
}

#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
    victory: bool,
    exit_timer: Option<Timer>,
}

// 🎬 LES SYSTÈMES (Les règles d'action du jeu)

// On prépare le monde au début
fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d); // On installe tes yeux (la caméra)
    
    // On fabrique ton vaisseau spatial
    commands.spawn((
        Player, Movable { velocity: Vec2::ZERO }, Health { current: PLAYER_HEALTH },
        Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() },
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));

    // On dessine le tableau de bord en haut
    commands.spawn(Node { width: Val::Percent(100.0), height: Val::Px(50.0), justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::horizontal(Val::Px(20.0)), ..default() })
        .with_children(|parent| {
            parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() }));
            parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() }));
            parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() }));
        });

    // Zone pour les gros messages au milieu
    commands.spawn((MainMessage, Text::new(""), TextFont { font_size: 40.0, ..default() }, Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() }));
}

// Système pour te déplacer
fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) {
    if let Ok(mut movable) = query.single_mut() {
        let mut dir = 0.0;
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; } // Gauche
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; } // Droite
        movable.velocity.x = dir * PLAYER_SPEED;
    }
}

// Système pour tirer tes lasers
fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) {
    if kb.just_pressed(KeyCode::Space) { // Espace pour tirer
        if let Ok(transform) = query.single() {
            commands.spawn((
                Bullet { from_player: true }, Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)),
            ));
        }
    }
}

// Système pour faire tirer les aliens
fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>, player_q: Query<&Transform, With<Player>>, mut enemy_q: Query<(&Transform, &mut EnemyFireTimer, &Enemy)>) {
    let player_pos = player_q.single().map(|t| t.translation.xy()).ok();
    let window = window_q.single().expect("Pas de fenêtre !");
    let width_limit = window.width() / 2.0;
    let height_limit = window.height() / 2.0;

    for (e_trans, mut timer, _) in enemy_q.iter_mut() {
        let e_pos = e_trans.translation.xy();
        let is_inside = e_pos.x.abs() < width_limit && e_pos.y.abs() < height_limit;
        timer.0.tick(time.delta()); // La montre de l'alien avance

        if timer.0.just_finished() && is_inside {
            let mut dir = Vec2::new(0.0, -1.0); // Tir vers le bas
            if let Some(p_pos) = player_pos { dir = (p_pos - e_pos).normalize(); } // Il te vise !
            commands.spawn((
                Bullet { from_player: false }, Movable { velocity: dir * (ENEMY_SPEED * 1.5) },
                Sprite { image: asset_server.load("sprites/bullet_02.png"), custom_size: Some(BULLET_SIZE), color: Color::srgb(1.0, 0.2, 0.2), ..default() },
                Transform::from_translation(e_trans.translation),
            ));
        }
    }
}

// Système qui gère tous les mouvements
fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().expect("Pas de fenêtre !");
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;

    for (entity, movable, mut transform) in query.iter_mut() {
        transform.translation += movable.velocity.extend(0.0) * time.delta_secs(); // On avance
        if transform.translation.y < -200.0 && transform.translation.y > -400.0 { // Bloquer le joueur sur les bords
            transform.translation.x = transform.translation.x.clamp(-half_w + 20.0, half_w - 20.0);
        }
        if transform.translation.y.abs() > half_h + 150.0 { commands.entity(entity).despawn(); } // Supprimer si trop loin
    }
}

// Système pour faire apparaître les vagues d'ennemis
fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, game_state: Res<GameState>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) {
    let window = window_q.single().expect("Pas de fenêtre !");
    let enemy_count = enemy_q.iter().count();

    match wave_mgr.state {
        WaveState::Spawning => {
            wave_mgr.spawn_timer.tick(time.delta());
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 {
                let is_boss = wave_mgr.enemies_spawned == 9; // Le 10ème est le Boss
                let (kind, size, sprite) = if is_boss { (EnemyType::Boss, ENEMY_SIZE * 2.5, "sprites/alien_red.png") } else { (EnemyType::Soldier, ENEMY_SIZE, "sprites/alien_grey.png") };
                let start_x = (rand::random::<f32>() - 0.5) * window.width() * 0.8;
                let start_pos = Vec3::new(start_x, window.height()/2.0 + 20.0, 0.0);
                
                commands.spawn((
                    Enemy { kind }, Movable { velocity: Vec2::new(0.0, -ENEMY_SPEED) },
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)),
                    Sprite { image: asset_server.load(sprite), custom_size: Some(size), ..default() },
                    Transform::from_translation(start_pos)
                ));
                wave_mgr.enemies_spawned += 1;
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; }
            }
        },
        WaveState::Fighting => if enemy_count == 0 {
            if wave_mgr.current_wave >= 5 { /* Fin de partie victorieuse */ }
            else { wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); }
        },
        WaveState::Waiting => {
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() {
                wave_mgr.enemies_spawned = 0; wave_mgr.enemies_killed = 0; wave_mgr.state = WaveState::Spawning;
            }
        }
    }
}

// 💥 LE SYSTÈME DE COLLISIONS (Quand ça fait BOUM)
fn collision_system(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform, &Enemy)>, mut player_q: Query<(&Transform, &mut Health), With<Player>>) {
    if let Ok((p_trans, mut p_health)) = player_q.single_mut() {
        let p_pos = p_trans.translation.xy();
        for (e_ent, e_trans, e_info) in enemy_q.iter() {
            let e_pos = e_trans.translation.xy();
            
            // Crash entre toi et un alien
            if p_pos.distance(e_pos) < 30.0 {
                commands.entity(e_ent).despawn(); // L'alien disparaît
                p_health.current -= 1; // Tu perds une vie
                if p_health.current <= 0 { state.game_over = true; }
            }

            // Quand tes balles touchent un alien
            for (b_ent, b_trans, b_type) in bullet_q.iter() {
                if b_type.from_player && b_trans.translation.xy().distance(e_pos) < 25.0 {
                    let pts = match e_info.kind { EnemyType::Boss => 50, EnemyType::Soldier => 10 };
                    state.score += pts;
                    wave_mgr.enemies_killed += 1; // On compte l'alien abattu
                    
                    // 🎆 ON RAJOUTE LE SPRITE D'EXPLOSION
                    commands.spawn((
                        Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) },
                        Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() },
                        Transform::from_translation(e_trans.translation),
                    ));

                    // 💰 ON RAJOUTE LES POINTS QUI S'AFFICHENT À CÔTÉ
                    commands.spawn((
                        FloatingText { timer: Timer::from_seconds(1.0, TimerMode::Once) },
                        Text::new(format!("+{}", pts)),
                        TextFont { font_size: 25.0, ..default() },
                        TextColor(if pts == 50 { Color::srgb(1.0, 1.0, 0.0) } else { Color::WHITE }),
                        Transform::from_translation(e_trans.translation + Vec3::new(35.0, 10.0, 1.0)), // Juste à droite !
                    ));

                    commands.entity(e_ent).despawn(); // Adieu l'alien
                    commands.entity(b_ent).despawn(); // Adieu la balle
                }
            }
        }
    }
}

// Système pour nettoyer les vieilles explosions et faire voler les points
fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosions: Query<(Entity, &mut Explosion)>, mut floating: Query<(Entity, &mut FloatingText, &mut Transform)>) {
    for (e, mut ex) in explosions.iter_mut() {
        ex.timer.tick(time.delta());
        if ex.timer.is_finished() { commands.entity(e).despawn(); }
    }
    for (e, mut ft, mut tr) in floating.iter_mut() {
        ft.timer.tick(time.delta());
        tr.translation.y += 1.0; // Les points montent au ciel
        if ft.timer.is_finished() { commands.entity(e).despawn(); }
    }
}

// Système d'affichage de l'interface (UI)
fn ui_system(wave_mgr: Res<WaveManager>, state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) {
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} Lvl: {}", wave_mgr.current_wave, wave_mgr.current_level); }
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", state.score); }
    let hp = player_q.single().map(|h| h.current).unwrap_or(0);
    for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp); }

    for mut t in texts.p3().iter_mut() {
        if state.game_over { **t = "GAME OVER".to_string(); }
        else if wave_mgr.state == WaveState::Waiting {
            if wave_mgr.current_wave == 1 { **t = format!("LEVEL {}", wave_mgr.current_level); }
            else if wave_mgr.enemies_killed == 10 { **t = "GOOD JOB !!!".to_string(); }
            else { **t = "".to_string(); }
        } else { **t = "".to_string(); }
    }
}

// 🚀 LE LANCEMENT (C'est parti !)
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1))) // Fond bleu foncé
        .init_resource::<GameState>()
        .init_resource::<WaveManager>()
        .add_systems(Startup, setup_game)
        .add_systems(Update, (
            player_control_system, player_shoot_system, enemy_shoot_system,
            movement_system, wave_system, collision_system, cleanup_system, ui_system
        ))
        .run();
}