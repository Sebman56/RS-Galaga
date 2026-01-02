// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 17h49
// ═══════════════════════════════════════════════════════════════════════════
//
//
// Manque tir vers le vaisseau
// Manque appuie sur X Q pour quitter
// Manque P pour pause
// ═══════════════════════════════════════════════════════════════════════════
// 🚀 XGALAGA : VERSION COMPATIBLE BEVY 0.15+ (MESSAGEWRITER)
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*; // Outils de base Bevy
use bevy::window::PrimaryWindow; // Gestion fenêtre
use bevy::app::AppExit; // Pour quitter
use bevy::math::Vec3Swizzles; // Calculs vecteurs

const PLAYER_SPEED: f32 = 500.0; // Vitesse joueur
const BULLET_SPEED: f32 = 700.0; // Vitesse laser
const ENEMY_SPEED: f32 = 120.0; // Vitesse alien
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0); // Taille joueur
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0); // Taille alien
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0); // Taille laser
const PLAYER_HEALTH: i32 = 3; // Vies

#[derive(Component)] struct Player; // Tag joueur
#[derive(Component)] struct Enemy { kind: EnemyType } // Tag alien
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss } // Types aliens
#[derive(Component)] struct Bullet { from_player: bool } // Tag projectile
#[derive(Component)] struct Movable { velocity: Vec2 } // Mouvement
#[derive(Component)] struct Health { current: i32 } // Points de vie
#[derive(Component)] struct EnemyFireTimer(Timer); // Chrono tir alien
#[derive(Component)] struct Explosion { timer: Timer } // Chrono explosion
#[derive(Component)] struct FloatingText { timer: Timer } // Chrono score

#[derive(Component)] struct ScoreText; // UI Score
#[derive(Component)] struct LevelText; // UI Niveau
#[derive(Component)] struct LivesText; // UI Vies
#[derive(Component)] struct MainMessage; // Message central

#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right } // Origines
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting } // États

#[derive(Resource)] struct WaveManager { // Gérant des vagues
    current_level: u32, // Niveau
    current_wave: u32, // Vague
    state: WaveState, // Phase
    direction: SpawnDirection, // Côté
    enemies_spawned: usize, // Compteur spawn
    spawn_timer: Timer, // Cadence spawn
    wave_timer: Timer, // Pause
    enemies_killed: usize, // Compteur kills
}

impl Default for WaveManager { // Init par défaut
    fn default() -> Self { // Valeurs
        Self { // Création
            current_level: 1, current_wave: 1, state: WaveState::Spawning,
            direction: SpawnDirection::Top, enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(2.5, TimerMode::Once),
            enemies_killed: 0,
        } // Fin
    } // Fin
}

#[derive(Resource, Default)] struct GameState { // État global
    score: u32, // Points
    game_over: bool, // Perdu
    victory: bool, // Gagné
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) { // Init
    commands.spawn(Camera2d); // Caméra
    spawn_player(&mut commands, &asset_server); // Créer joueur
    commands.spawn(Node { width: Val::Percent(100.0), height: Val::Px(50.0), justify_content: JustifyContent::SpaceBetween, ..default() }) // Barre UI
        .with_children(|parent| { // Enfants
            parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() })); // Lvl
            parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() })); // Score
            parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() })); // Vies
        }); // Fin
    commands.spawn((MainMessage, Text::new(""), TextFont { font_size: 40.0, ..default() }, Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() })); // Msg
}

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) { // Spawn joueur
    commands.spawn(( // Joueur
        Player, Movable { velocity: Vec2::ZERO }, Health { current: PLAYER_HEALTH }, // Tags
        Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() }, // Image
        Transform::from_xyz(0.0, -300.0, 1.0), // Pos
    )); // Fin
}

// 🛠️ CORRECTION ICI : Utilisation de MessageWriter et .write()
fn input_system(kb: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>, mut game_state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, mut commands: Commands, asset_server: Res<AssetServer>, entities: Query<Entity, Or<(With<Player>, With<Enemy>, With<Bullet>, With<Explosion>)>>) { // Touches
    if kb.just_pressed(KeyCode::KeyQ) || kb.just_pressed(KeyCode::KeyX) { exit.write(AppExit::Success); } // Quitter corrigé
    if kb.just_pressed(KeyCode::KeyR) { // Restart
        for e in entities.iter() { commands.entity(e).despawn(); } // Nettoyer
        *game_state = GameState::default(); // Reset score
        *wave_mgr = WaveManager::default(); // Reset vagues
        spawn_player(&mut commands, &asset_server); // Revivre
    } // Fin
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) { // Mouvement
    if let Ok(mut movable) = query.single_mut() { // Si joueur
        let mut dir = 0.0; // Axe X
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; } // Gauche
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; } // Droite
        movable.velocity.x = dir * PLAYER_SPEED; // Vitesse
    } // Fin
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) { // Tir
    if kb.just_pressed(KeyCode::Space) { // Espace
        if let Ok(transform) = query.single() { // Vivant
            commands.spawn(( // Balle
                Bullet { from_player: true }, Movable { velocity: Vec2::new(0.0, BULLET_SPEED) }, // Tags
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() }, // Image
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)), // Pos
            )); // Fin
        } // Fin
    } // Fin
}

fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut enemy_q: Query<(&Transform, &mut EnemyFireTimer)>) { // Tir alien
    for (trans, mut timer) in enemy_q.iter_mut() { // Loop mut
        timer.0.tick(time.delta()); // Horloge
        if timer.0.just_finished() { // Feu
            commands.spawn(( // Laser rouge
                Bullet { from_player: false }, Movable { velocity: Vec2::new(0.0, -ENEMY_SPEED * 2.0) }, // Vitesse
                Sprite { image: asset_server.load("sprites/bullet_02.png"), custom_size: Some(BULLET_SIZE), color: Color::srgb(1.0, 0.2, 0.2), ..default() }, // Couleur
                Transform::from_translation(trans.translation), // Pos
            )); // Fin
        } // Fin
    } // Fin
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>) { // Physique
    let window = window_q.single().expect("Window error"); // Fenêtre
    let (hw, hh) = (window.width()/2.0, window.height()/2.0); // Bords
    for (entity, movable, mut trans) in query.iter_mut() { // Boucle
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs(); // Update
        if trans.translation.y < -200.0 { trans.translation.x = trans.translation.x.clamp(-hw + 20.0, hw - 20.0); } // Limite joueur
        if trans.translation.y.abs() > hh + 100.0 || trans.translation.x.abs() > hw + 100.0 { commands.entity(entity).despawn(); } // Nettoyage
    } // Fin
}

fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, mut game_state: ResMut<GameState>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) { // Vagues
    let window = window_q.single().expect("Window error"); // Fenêtre
    let (hw, hh) = (window.width()/2.0, window.height()/2.0); // Bords
    let enemy_count = enemy_q.iter().count(); // Aliens restants
    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) { // Direction
        (1, 3) => SpawnDirection::Right, (1, 4) => SpawnDirection::Left, 
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right, (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left, 
        _ => SpawnDirection::Top, 
    }; // Fin
    match wave_mgr.state { // Phase
        WaveState::Spawning => { // Création
            wave_mgr.spawn_timer.tick(time.delta()); // Horloge
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 { // Nouveau
                let is_boss = wave_mgr.enemies_spawned == 9; // Boss ?
                let sprite = if is_boss { "sprites/alien_red.png" } else { match wave_mgr.direction { // Image
                    SpawnDirection::Left => "sprites/alien_red.png", SpawnDirection::Right => "sprites/alien_green.png", _ => "sprites/alien_grey.png" 
                }}; // Fin
                let (start, vel) = match wave_mgr.direction { // Physique départ
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, hh + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)), 
                    SpawnDirection::Left => (Vec3::new(-hw - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)), 
                    SpawnDirection::Right => (Vec3::new(hw + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)), 
                }; // Fin
                commands.spawn((Enemy { kind: if is_boss { EnemyType::Boss } else { EnemyType::Soldier } }, Movable { velocity: vel }, EnemyFireTimer(Timer::from_seconds(if is_boss { 1.0 } else { 2.5 }, TimerMode::Repeating)), Sprite { image: asset_server.load(sprite), custom_size: Some(if is_boss { ENEMY_SIZE * 2.5 } else { ENEMY_SIZE }), ..default() }, Transform::from_translation(start))); // Spawn
                wave_mgr.enemies_spawned += 1; // +1
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; } // Combat
            } // Fin chrono
        }, // Fin Spawning
        WaveState::Fighting => if enemy_count == 0 { // Fini ?
            if wave_mgr.current_wave >= 5 { if wave_mgr.current_level >= 3 { game_state.victory = true; } else { wave_mgr.current_level += 1; wave_mgr.current_wave = 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } }
            else { wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } // Next
        }, // Fin Fighting
        WaveState::Waiting => { wave_mgr.wave_timer.tick(time.delta()); if wave_mgr.wave_timer.is_finished() && !game_state.victory { wave_mgr.enemies_spawned = 0; wave_mgr.enemies_killed = 0; wave_mgr.state = WaveState::Spawning; } } // Reset
    } // Fin match
} // Fin

fn collision_system(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform, &Enemy)>, mut player_q: Query<(Entity, &Transform, &mut Health), With<Player>>) { // Chocs
    if let Ok((p_ent, p_trans, mut p_health)) = player_q.single_mut() { // Joueur ?
        for (e_ent, e_trans, e_info) in enemy_q.iter() { // Loop aliens
            if p_trans.translation.xy().distance(e_trans.translation.xy()) < 25.0 { // Choc joueur
                commands.spawn((Explosion { timer: Timer::from_seconds(0.4, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(50.0, 50.0)), ..default() }, Transform::from_translation(p_trans.translation))); // Boom
                commands.entity(e_ent).despawn(); p_health.current -= 1; // Dégâts
                if p_health.current <= 0 { commands.entity(p_ent).despawn(); state.game_over = true; } // Mort
            } // Fin
            for (b_ent, b_trans, b_type) in bullet_q.iter() { // Loop balles
                let dist = b_trans.translation.xy().distance(e_trans.translation.xy()); // Distance
                if b_type.from_player && dist < 25.0 { // Hit alien
                    let pts = if e_info.kind == EnemyType::Boss { 50 } else { 10 }; // Score
                    state.score += pts; wave_mgr.enemies_killed += 1; // Update
                    commands.spawn((Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() }, Transform::from_translation(e_trans.translation))); // Boom
                    commands.spawn((FloatingText { timer: Timer::from_seconds(1.0, TimerMode::Once) }, Text2d::new(format!("+{}", pts)), TextFont { font_size: 30.0, ..default() }, TextColor(Color::WHITE), Transform::from_translation(e_trans.translation + Vec3::new(0.0, 20.0, 2.0)))); // Score
                    commands.entity(e_ent).despawn(); commands.entity(b_ent).despawn(); // Kill
                } // Fin hit alien
                if !b_type.from_player && b_trans.translation.xy().distance(p_trans.translation.xy()) < 20.0 { // Hit joueur
                    commands.spawn((Explosion { timer: Timer::from_seconds(0.4, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(50.0, 50.0)), ..default() }, Transform::from_translation(p_trans.translation))); // Boom joueur
                    commands.entity(b_ent).despawn(); p_health.current -= 1; // Dégâts
                    if p_health.current <= 0 { commands.entity(p_ent).despawn(); state.game_over = true; } // Mort
                } // Fin hit joueur
            } // Fin
        } // Fin
    } // Fin
} // Fin

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosions: Query<(Entity, &mut Explosion)>, mut floating: Query<(Entity, &mut FloatingText, &mut Transform)>) { // Clean
    for (e, mut ex) in explosions.iter_mut() { ex.timer.tick(time.delta()); if ex.timer.is_finished() { commands.entity(e).despawn(); } } // Explode
    for (e, mut ft, mut tr) in floating.iter_mut() { ft.timer.tick(time.delta()); tr.translation.y += 1.5; if ft.timer.is_finished() { commands.entity(e).despawn(); } } // Score
} // Fin

fn ui_system(wave_mgr: Res<WaveManager>, state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) { // UI
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} Lvl: {}", wave_mgr.current_wave, wave_mgr.current_level); } // Text Lvl
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", state.score); } // Text Score
    let hp = if let Ok(h) = player_q.single() { h.current } else { 0 }; // Vies
    for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp); } // Update Vies
    for mut t in texts.p3().iter_mut() { // Msg Central
        if state.game_over { **t = "GAME OVER (R pour rejouer)".to_string(); } // Perdu
        else if state.victory { **t = "VICTOIRE TOTALE !".to_string(); } // Gagné
        else if wave_mgr.state == WaveState::Waiting { if wave_mgr.current_wave == 1 { **t = format!("LEVEL {}", wave_mgr.current_level); } else if wave_mgr.enemies_killed == 10 { **t = "GoodJob !!!".to_string(); } else { **t = "".to_string(); } }
        else { **t = "".to_string(); } // Rien
    } // Fin
} // Fin

fn main() { // Main
    App::new() // App
        .add_plugins(DefaultPlugins) // Plugins
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1))) // Fond
        .init_resource::<GameState>() // Score
        .init_resource::<WaveManager>() // Vagues
        .add_systems(Startup, setup_game) // Start
        .add_systems(Update, (input_system, player_control_system, player_shoot_system, enemy_shoot_system, movement_system, wave_system, collision_system, cleanup_system, ui_system)) // Loop
        .run(); // Play
} // Fin