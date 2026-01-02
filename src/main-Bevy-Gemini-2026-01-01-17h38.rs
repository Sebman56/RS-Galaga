// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 17h38
// ═══════════════════════════════════════════════════════════════════════════
//
//// ═══════════════════════════════════════════════════════════════════════════
// 🚀 XGALAGA : VERSION SPÉCIALE MESSAGES ET RÉCOMPENSES
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*; // Importation des outils Bevy
use bevy::window::PrimaryWindow; // Outil pour la gestion de la fenêtre
use bevy::app::AppExit; // Outil pour quitter le jeu
use bevy::math::Vec3Swizzles; // Astuce pour les vecteurs (X, Y)

const PLAYER_SPEED: f32 = 500.0; // Vitesse du vaisseau joueur
const BULLET_SPEED: f32 = 700.0; // Vitesse des lasers
const ENEMY_SPEED: f32 = 120.0; // Vitesse des aliens
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0); // Taille du joueur
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0); // Taille des aliens
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0); // Taille des projectiles
const PLAYER_HEALTH: i32 = 3; // Nombre de vies

#[derive(Component)] struct Player; // Marqueur joueur
#[derive(Component)] struct Enemy { kind: EnemyType } // Marqueur ennemi avec son type
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss } // Types d'ennemis
#[derive(Component)] struct Bullet { from_player: bool } // Marqueur laser
#[derive(Component)] struct Movable { velocity: Vec2 } // Composant de mouvement
#[derive(Component)] struct Health { current: i32 } // Composant de vie
#[derive(Component)] struct EnemyFireTimer(Timer); // Chrono de tir ennemi
#[derive(Component)] struct Explosion { timer: Timer } // Chrono d'explosion
#[derive(Component)] struct FloatingText { timer: Timer } // Chrono score flottant

#[derive(Component)] struct ScoreText; // Marqueur texte score
#[derive(Component)] struct LevelText; // Marqueur texte niveau
#[derive(Component)] struct LivesText; // Marqueur texte vies
#[derive(Component)] struct MainMessage; // Marqueur message central

#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right } // Origines
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting } // États

#[derive(Resource)] struct WaveManager { // Gestionnaire de progression
    current_level: u32, // Niveau actuel
    current_wave: u32, // Vague actuelle
    state: WaveState, // Phase de la vague
    direction: SpawnDirection, // Direction d'arrivée
    enemies_spawned: usize, // Compteur d'apparition
    spawn_timer: Timer, // Rythme d'apparition
    wave_timer: Timer, // Temps de pause
    enemies_killed: usize, // Compteur de destruction réelle
}

impl Default for WaveManager { // Initialisation par défaut
    fn default() -> Self { // Valeurs de départ
        Self { // Création
            current_level: 1, // Start Level 1
            current_wave: 1, // Start Wave 1
            state: WaveState::Spawning, // Start Spawning
            direction: SpawnDirection::Top, // Start Top
            enemies_spawned: 0, // Zéro apparu
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating), // Cadence
            wave_timer: Timer::from_seconds(2.5, TimerMode::Once), // Pause
            enemies_killed: 0, // Zéro tué
        } // Fin
    } // Fin
}

#[derive(Resource, Default)] struct GameState { // État du jeu
    score: u32, // Score joueur
    game_over: bool, // État perdu
    victory: bool, // État gagné
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) { // Démarrage
    commands.spawn(Camera2d); // Caméra
    commands.spawn(( // Joueur
        Player, // Tag
        Movable { velocity: Vec2::ZERO }, // Vitesse
        Health { current: PLAYER_HEALTH }, // Vies
        Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() }, // Image
        Transform::from_xyz(0.0, -300.0, 0.0), // Position
    )); // Fin joueur
    commands.spawn(Node { width: Val::Percent(100.0), height: Val::Px(50.0), justify_content: JustifyContent::SpaceBetween, ..default() }) // UI
        .with_children(|parent| { // Enfants UI
            parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() })); // Niveau
            parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() })); // Score
            parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() })); // Vies
        }); // Fin UI
    commands.spawn((MainMessage, Text::new(""), TextFont { font_size: 40.0, ..default() }, Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() })); // Message
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) { // Contrôles
    if let Ok(mut movable) = query.single_mut() { // Joueur ?
        let mut dir = 0.0; // Direction X
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; } // Gauche
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; } // Droite
        movable.velocity.x = dir * PLAYER_SPEED; // Vitesse finale
    } // Fin
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) { // Tirs
    if kb.just_pressed(KeyCode::Space) { // Espace ?
        if let Ok(transform) = query.single() { // Vivant ?
            commands.spawn(( // Balle
                Bullet { from_player: true }, // Tag
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) }, // Montée
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() }, // Image
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)), // Position
            )); // Fin balle
        } // Fin
    } // Fin
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>) { // Mouvements
    let window = window_q.single().expect("Window error"); // Fenêtre
    let (hw, hh) = (window.width()/2.0, window.height()/2.0); // Marges
    for (entity, movable, mut trans) in query.iter_mut() { // Boucle
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs(); // Update
        if trans.translation.y < -200.0 { trans.translation.x = trans.translation.x.clamp(-hw + 20.0, hw - 20.0); } // Mur joueur
        if trans.translation.y.abs() > hh + 100.0 || trans.translation.x.abs() > hw + 100.0 { commands.entity(entity).despawn(); } // Poubelle
    } // Fin
}

fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, mut game_state: ResMut<GameState>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) { // Vagues
    let window = window_q.single().expect("Window error"); // Fenêtre
    let enemy_count = enemy_q.iter().count(); // Aliens vivants
    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) { // Direction logic
        (1, 3) => SpawnDirection::Right, (1, 4) => SpawnDirection::Left, // Lvl 1
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right, // Lvl 2/3
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left, // Lvl 2/3
        _ => SpawnDirection::Top, // Défaut
    }; // Fin
    match wave_mgr.state { // Phase ?
        WaveState::Spawning => { // Création
            wave_mgr.spawn_timer.tick(time.delta()); // Horloge
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 { // Spawn
                let is_boss = wave_mgr.enemies_spawned == 9; // Boss check
                let sprite_path = if is_boss { "sprites/alien_red.png" } else { match wave_mgr.direction { // Sprite
                    SpawnDirection::Left => "sprites/alien_red.png", // Rouge
                    SpawnDirection::Right => "sprites/alien_green.png", // Vert
                    SpawnDirection::Top => "sprites/alien_grey.png", // Gris
                }}; // Fin
                let (start_pos, velocity) = match wave_mgr.direction { // Pos/Vel
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)), // Top
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)), // Left
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)), // Right
                }; // Fin
                commands.spawn(( // Alien
                    Enemy { kind: if is_boss { EnemyType::Boss } else { EnemyType::Soldier } }, // Type
                    Movable { velocity }, // Vitesse
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)), // Laser
                    Sprite { image: asset_server.load(sprite_path), custom_size: Some(if is_boss { ENEMY_SIZE * 2.5 } else { ENEMY_SIZE }), ..default() }, // Apparence
                    Transform::from_translation(start_pos) // Spawn
                )); // Fin
                wave_mgr.enemies_spawned += 1; // Incrément
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; } // Combat mode
            } // Fin spawn
        }, // Fin
        WaveState::Fighting => if enemy_count == 0 { // Nettoyé ?
            if wave_mgr.current_wave >= 5 { // Fin vague 5 ?
                if wave_mgr.current_level >= 3 { game_state.victory = true; } // GAGNER
                else { wave_mgr.current_level += 1; wave_mgr.current_wave = 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } // Next Level
            } else { wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } // Next Wave
        }, // Fin
        WaveState::Waiting => { // Pause
            wave_mgr.wave_timer.tick(time.delta()); // Horloge pause
            if wave_mgr.wave_timer.is_finished() && !game_state.victory { // Fini ?
                wave_mgr.enemies_spawned = 0; // Reset spawn
                wave_mgr.enemies_killed = 0; // Reset kill count pour le message GoodJob
                wave_mgr.state = WaveState::Spawning; // Relance
            } // Fin
        } // Fin
    } // Fin
} // Fin

fn collision_system(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform, &Enemy)>, mut player_q: Query<(&Transform, &mut Health), With<Player>>) { // Collisions
    if let Ok((p_trans, mut p_health)) = player_q.single_mut() { // Joueur vivant ?
        for (e_ent, e_trans, e_info) in enemy_q.iter() { // Loop aliens
            if p_trans.translation.xy().distance(e_trans.translation.xy()) < 25.0 { // Choc joueur ?
                commands.entity(e_ent).despawn(); // Mort alien
                p_health.current -= 1; // -1 vie
                if p_health.current <= 0 { state.game_over = true; } // Mort totale
            } // Fin
            for (b_ent, b_trans, b_type) in bullet_q.iter() { // Loop lasers
                if b_type.from_player && b_trans.translation.xy().distance(e_trans.translation.xy()) < 25.0 { // Hit ?
                    let pts = match e_info.kind { EnemyType::Boss => 50, EnemyType::Soldier => 10 }; // Score
                    state.score += pts; // Ajout
                    wave_mgr.enemies_killed += 1; // +1 kill réel
                    commands.spawn((Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() }, Transform::from_translation(e_trans.translation))); // Boom
                    commands.spawn((FloatingText { timer: Timer::from_seconds(1.0, TimerMode::Once) }, Text2d::new(format!("+{}", pts)), TextFont { font_size: 30.0, ..default() }, TextColor(if pts == 50 { Color::srgb(1.0, 1.0, 0.0) } else { Color::WHITE }), Transform::from_translation(e_trans.translation + Vec3::new(20.0, 10.0, 2.0)))); // Pop points
                    commands.entity(e_ent).despawn(); // Bye alien
                    commands.entity(b_ent).despawn(); // Bye laser
                } // Fin
            } // Fin lasers
        } // Fin aliens
    } // Fin joueur
} // Fin collisions

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosions: Query<(Entity, &mut Explosion)>, mut floating: Query<(Entity, &mut FloatingText, &mut Transform)>) { // Nettoyage
    for (e, mut ex) in explosions.iter_mut() { ex.timer.tick(time.delta()); if ex.timer.is_finished() { commands.entity(e).despawn(); } } // Nettoyage explosions
    for (e, mut ft, mut tr) in floating.iter_mut() { ft.timer.tick(time.delta()); tr.translation.y += 1.5; if ft.timer.is_finished() { commands.entity(e).despawn(); } } // Nettoyage scores
} // Fin

fn ui_system(wave_mgr: Res<WaveManager>, state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) { // UI
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} Lvl: {}", wave_mgr.current_wave, wave_mgr.current_level); } // Text Vague
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", state.score); } // Text Score
    if let Ok(hp) = player_q.single() { for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp.current); } } // Text Vies
    for mut t in texts.p3().iter_mut() { // Text Central
        if state.game_over { **t = "GAME OVER".to_string(); } // Game Over
        else if state.victory { **t = "VICTOIRE TOTALE !".to_string(); } // Victoire
        else if wave_mgr.state == WaveState::Waiting { // Pendant la pause
            if wave_mgr.current_wave == 1 { **t = format!("LEVEL {}", wave_mgr.current_level); } // Affichage LEVEL X au changement
            else if wave_mgr.enemies_killed == 10 { **t = "GoodJob !!!".to_string(); } // GoodJob si tous tués
            else { **t = "".to_string(); } // Sinon rien
        } // Fin pause
        else { **t = "".to_string(); } // En jeu -> rien
    } // Fin text central
} // Fin UI

fn main() { // Main
    App::new() // App
        .add_plugins(DefaultPlugins) // Plugins
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1))) // Fond
        .init_resource::<GameState>() // Init Score
        .init_resource::<WaveManager>() // Init Vagues
        .add_systems(Startup, setup_game) // Setup
        .add_systems(Update, (player_control_system, player_shoot_system, movement_system, wave_system, collision_system, cleanup_system, ui_system)) // Loop
        .run(); // Play
} // Fin