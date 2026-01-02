
// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 22h53
// ═══════════════════════════════════════════════════════════════════════════
//
//
// Message correct
// le vaisseau ok
// Utilisation des touches "Q" "X" pour sortir, "P" pour pause, "R" pour recommencer le jeu
//
//
//
// Tous les aliens sont les les mêmes
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::app::AppExit;

// ==========================================
// 🛠️ LES RÉGLAGES
// ==========================================
const PLAYER_SPEED: f32 = 500.0;
const BULLET_SPEED: f32 = 700.0;
const ENEMY_SPEED: f32 = 120.0;
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0);
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0);
const PLAYER_HEALTH: i32 = 3;

// ==========================================
// 🏷️ LES ÉTIQUETTES ET ÉTATS
// ==========================================
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppState { #[default] Running, Paused }

#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy { kind: EnemyType }
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss }
#[derive(Component)] struct Bullet { from_player: bool }
#[derive(Component)] struct Movable { velocity: Vec2 }
#[derive(Component)] struct Health { current: i32 }
#[derive(Component)] struct EnemyFireTimer(Timer);
#[derive(Component)] struct Explosion { timer: Timer }
#[derive(Component)] struct FloatingScore { timer: Timer }

#[derive(Component)] struct ScoreText;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct LivesText;
#[derive(Component)] struct MainMessage;

#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right }
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, LevelCompleted, Waiting }

#[derive(Resource)]
struct WaveManager {
    current_level: u32,
    current_wave: u32,
    state: WaveState,
    direction: SpawnDirection,
    enemies_spawned: usize,
    enemies_killed_by_player: usize,
    spawn_timer: Timer,
    wave_timer: Timer,
    show_good_job: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_level: 1,
            current_wave: 1,
            state: WaveState::Spawning,
            direction: SpawnDirection::Top,
            enemies_spawned: 0,
            enemies_killed_by_player: 0,
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(2.0, TimerMode::Once),
            show_good_job: false,
        }
    }
}

#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
    victory: bool,
}

// ==========================================
// 🚀 LE LANCEMENT
// ==========================================
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .init_resource::<GameState>()
        .init_resource::<WaveManager>()
        .init_state::<AppState>() // On initialise l'état de pause
        .add_systems(Startup, setup_game)
        .add_systems(Update, (
            input_system, // Toujours actif (pour pause/quit/reset)
            ui_update_system,
        ))
        // Ces systèmes ne tournent que si le jeu n'est pas en pause
        .add_systems(Update, (
            player_control_system, player_shoot_system,
            enemy_shoot_system, movement_system, wave_system,
            collision_system, cleanup_system
        ).run_if(in_state(AppState::Running)))
        .run();
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    spawn_player(&mut commands, &asset_server);

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::all(Val::Px(15.0)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((LevelText, Text::new(""), TextFont::from_font_size(20.0)));
        parent.spawn((ScoreText, Text::new("Score: 0"), TextFont::from_font_size(25.0)));
        parent.spawn((LivesText, Text::new("Vies: 3"), TextFont::from_font_size(20.0)));
    });

    commands.spawn((
        MainMessage,
        Text::new(""),
        TextFont::from_font_size(50.0),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(45.0),
            ..default()
        }
    ));
}

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn((
        Player,
        Movable { velocity: Vec2::ZERO },
        Health { current: PLAYER_HEALTH },
        Sprite {
            image: asset_server.load("sprites/player_01.png"),
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 1.0),
    ));
}

// ==========================================
// 🕹️ SYSTÈME D'ENTRÉE (Quit, Pause, Reset)
// ==========================================
fn input_system(
    kb: Res<ButtonInput<KeyCode>>, 
    mut exit: MessageWriter<AppExit>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut wave_mgr: ResMut<WaveManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Pour le nettoyage au Reset :
    entities_q: Query<Entity, Or<(With<Enemy>, With<Bullet>, With<Player>, With<Explosion>)>>,
) {
    // QUITTER : Échap, Q ou X
    if kb.just_pressed(KeyCode::Escape) || kb.just_pressed(KeyCode::KeyQ) || kb.just_pressed(KeyCode::KeyX) {
        exit.write(AppExit::Success);
    }

    // PAUSE : Touche P
    if kb.just_pressed(KeyCode::KeyP) {
        match state.get() {
            AppState::Running => next_state.set(AppState::Paused),
            AppState::Paused => next_state.set(AppState::Running),
        }
    }

    // RECOMMENCER : Touche R
    if kb.just_pressed(KeyCode::KeyR) {
        // 1. Reset des ressources
        *game_state = GameState::default();
        *wave_mgr = WaveManager::default();
        // 2. Supprimer les entités existantes
        for entity in entities_q.iter() {
            commands.entity(entity).despawn();
        }
        // 3. Recréer le joueur et s'assurer que l'état est "Running"
        spawn_player(&mut commands, &asset_server);
        next_state.set(AppState::Running);
    }
}

// ==========================================
// 👾 LOGIQUE DE JEU (Extraits identiques)
// ==========================================

fn wave_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    time: Res<Time>, 
    mut wave_mgr: ResMut<WaveManager>, 
    mut game_state: ResMut<GameState>, 
    enemy_q: Query<&Enemy>, 
    window_q: Query<&Window, With<PrimaryWindow>>
) {
    let Ok(window) = window_q.single() else { return };
    let enemy_count = enemy_q.iter().count();

    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) {
        (1, 3) => SpawnDirection::Right, 
        (1, 4) => SpawnDirection::Left,
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right,
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left,
        _ => SpawnDirection::Top,
    };

    match wave_mgr.state {
        WaveState::Spawning => {
            wave_mgr.spawn_timer.tick(time.delta());
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 {
                let is_boss = wave_mgr.enemies_spawned == 9;
                let (start_pos, velocity) = match wave_mgr.direction {
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)),
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)),
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)),
                };

                commands.spawn((
                    Enemy { kind: if is_boss { EnemyType::Boss } else { EnemyType::Soldier } },
                    Movable { velocity },
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)),
                    Sprite { image: asset_server.load("sprites/alien_grey.png"), custom_size: Some(if is_boss { ENEMY_SIZE * 2.5 } else { ENEMY_SIZE }), ..default() },
                    Transform::from_translation(start_pos)
                ));
                wave_mgr.enemies_spawned += 1;
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; }
            }
        },
        WaveState::Fighting => {
            if enemy_count == 0 {
                wave_mgr.show_good_job = wave_mgr.enemies_killed_by_player >= 10;
                if wave_mgr.current_wave >= 5 {
                    if wave_mgr.current_level >= 3 { game_state.victory = true; }
                    else { 
                        wave_mgr.state = WaveState::LevelCompleted; 
                        wave_mgr.wave_timer.reset(); 
                    }
                } else { 
                    wave_mgr.current_wave += 1; 
                    wave_mgr.state = WaveState::Waiting; 
                    wave_mgr.wave_timer.reset(); 
                }
            }
        },
        WaveState::LevelCompleted => {
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() {
                wave_mgr.current_level += 1;
                wave_mgr.current_wave = 1;
                wave_mgr.enemies_spawned = 0;
                wave_mgr.enemies_killed_by_player = 0;
                wave_mgr.state = WaveState::Spawning;
            }
        }
        WaveState::Waiting => {
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() {
                wave_mgr.enemies_spawned = 0;
                wave_mgr.enemies_killed_by_player = 0;
                wave_mgr.state = WaveState::Spawning;
            }
        },
    }
}

fn player_control_system(
    kb: Res<ButtonInput<KeyCode>>, 
    window_q: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Movable, &mut Transform), With<Player>>, 
    state: Res<GameState>
) {
    if state.game_over || state.victory { return; }
    let Ok(window) = window_q.single() else { return };
    let limit = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;

    if let Ok((mut movable, mut trans)) = query.single_mut() {
        let mut dir = 0.0;
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; }
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; }
        movable.velocity.x = dir * PLAYER_SPEED;

        if trans.translation.x < -limit { trans.translation.x = -limit; movable.velocity.x = 0.0; }
        if trans.translation.x > limit { trans.translation.x = limit; movable.velocity.x = 0.0; }
    }
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>, state: Res<GameState>) {
    if state.game_over || state.victory { return; }
    if kb.just_pressed(KeyCode::Space) {
        if let Ok(transform) = query.single() {
            commands.spawn((
                Bullet { from_player: true },
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)),
            ));
        }
    }
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>) {
    let Ok(window) = window_q.single() else { return };
    let (hw, hh) = (window.width()/2.0, window.height()/2.0);
    for (entity, movable, mut trans) in query.iter_mut() {
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs();
        if trans.translation.y.abs() > hh + 100.0 || trans.translation.x.abs() > hw + 100.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut enemy_q: Query<(&Transform, &mut EnemyFireTimer)>, player_q: Query<&Transform, With<Player>>) {
    let Ok(p_trans) = player_q.single() else { return };
    for (e_trans, mut timer) in enemy_q.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            let dir = (p_trans.translation - e_trans.translation).xy().normalize_or_zero();
            commands.spawn((
                Bullet { from_player: false },
                Movable { velocity: dir * (ENEMY_SPEED * 1.8) },
                Sprite { image: asset_server.load("sprites/bullet_02.png"), custom_size: Some(BULLET_SIZE), color: Color::srgb(1.0, 0.0, 0.0), ..default() },
                Transform::from_translation(e_trans.translation),
            ));
        }
    }
}

fn collision_system(
    mut commands: Commands, 
    mut state: ResMut<GameState>, 
    mut wave_mgr: ResMut<WaveManager>,
    bullet_q: Query<(Entity, &Transform, &Bullet)>, 
    enemy_q: Query<(Entity, &Transform, &Enemy)>, 
    mut player_q: Query<(Entity, &Transform, &mut Health), With<Player>>,
    asset_server: Res<AssetServer>,
) {
    let Ok((p_ent, p_trans, mut p_health)) = player_q.single_mut() else { return };
    let p_pos = p_trans.translation.xy();

    for (e_ent, e_trans, e_info) in enemy_q.iter() {
        let e_pos = e_trans.translation.xy();
        let hit_radius = if e_info.kind == EnemyType::Boss { 50.0 } else { 25.0 };

        if p_pos.distance(e_pos) < hit_radius {
            commands.entity(e_ent).despawn();
            p_health.current -= 1;
            spawn_explosion(&mut commands, &asset_server, p_trans.translation);
            if p_health.current <= 0 { commands.entity(p_ent).despawn(); state.game_over = true; }
        }

        for (b_ent, b_trans, b_type) in bullet_q.iter() {
            let b_pos = b_trans.translation.xy();
            if b_type.from_player && b_pos.distance(e_pos) < hit_radius {
                let points = if e_info.kind == EnemyType::Boss { 100 } else { 10 };
                state.score += points;
                wave_mgr.enemies_killed_by_player += 1;
                spawn_explosion(&mut commands, &asset_server, e_trans.translation);
                
                commands.spawn((
                    FloatingScore { timer: Timer::from_seconds(0.7, TimerMode::Once) },
                    Text2d::new(format!("+{}", points)),
                    TextFont::from_font_size(22.0),
                    Transform::from_translation(e_trans.translation + Vec3::new(0.0, 20.0, 1.0)),
                ));

                commands.entity(e_ent).despawn();
                commands.entity(b_ent).despawn();
            } else if !b_type.from_player && b_pos.distance(p_pos) < 15.0 {
                p_health.current -= 1;
                spawn_explosion(&mut commands, &asset_server, p_trans.translation);
                commands.entity(b_ent).despawn();
                if p_health.current <= 0 { state.game_over = true; }
            }
        }
    }
}

fn spawn_explosion(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec3) {
    commands.spawn((
        Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) },
        Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::splat(60.0)), ..default() },
        Transform::from_translation(pos),
    ));
}

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosion_q: Query<(Entity, &mut Explosion)>, mut score_q: Query<(Entity, &mut FloatingScore, &mut Transform)>) {
    for (entity, mut explosion) in explosion_q.iter_mut() {
        explosion.timer.tick(time.delta());
        if explosion.timer.just_finished() { commands.entity(entity).despawn(); }
    }
    for (entity, mut score, mut trans) in score_q.iter_mut() {
        score.timer.tick(time.delta());
        trans.translation.y += 1.5;
        if score.timer.just_finished() { commands.entity(entity).despawn(); }
    }
}

// ==========================================
// 📺 MISE À JOUR DE L'ÉCRAN
// ==========================================
fn ui_update_system(
    state: Res<GameState>, 
    wave_mgr: Res<WaveManager>, 
    app_state: Res<State<AppState>>,
    player_q: Query<&Health, With<Player>>, 
    mut text_queries: ParamSet<(
        Query<&mut Text, With<ScoreText>>, 
        Query<&mut Text, With<LevelText>>, 
        Query<&mut Text, With<LivesText>>, 
        Query<&mut Text, With<MainMessage>>
    )>
) {
    if let Ok(mut text) = text_queries.p0().single_mut() { text.0 = format!("Score: {}", state.score); }
    if let Ok(mut text) = text_queries.p1().single_mut() { text.0 = format!("Lvl: {} Wv: {}", wave_mgr.current_level, wave_mgr.current_wave); }
    
    let hp = player_q.single().map(|h| h.current).unwrap_or(0);
    if let Ok(mut text) = text_queries.p2().single_mut() { text.0 = format!("Vies: {}", hp); }
    
    if let Ok(mut text) = text_queries.p3().single_mut() {
        // Priorité des messages : Pause > Game Over > Victoire > Level > Good Job
        if *app_state.get() == AppState::Paused { text.0 = "PAUSE".to_string(); }
        else if state.game_over { text.0 = "GAME OVER".to_string(); }
        else if state.victory { text.0 = "VICTOIRE TOTALE !".to_string(); }
        else if wave_mgr.state == WaveState::LevelCompleted { text.0 = format!("LEVEL {} RÉUSSI !", wave_mgr.current_level); }
        else if wave_mgr.state == WaveState::Waiting && wave_mgr.show_good_job { text.0 = "Good Job !!!".to_string(); }
        else { text.0 = "".to_string(); }
    }
}