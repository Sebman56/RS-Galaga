// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 16h45
// ═══════════════════════════════════════════════════════════════════════════
// ═══════════════════════════════════════════════════════════════════════════
// 🚀 XGALAGA : VERSION RÉPARÉE (Les vagues s'enchaînent enfin !)
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::app::AppExit;
use bevy::math::Vec3Swizzles;

// 🎮 RÉGLAGES
const PLAYER_SPEED: f32 = 500.0;
const BULLET_SPEED: f32 = 700.0;
const ENEMY_SPEED: f32 = 120.0;
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0);
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0);
const PLAYER_HEALTH: i32 = 3;

// 📦 COMPOSANTS
#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy { kind: EnemyType }
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss }
#[derive(Component)] struct Bullet { from_player: bool }
#[derive(Component)] struct Movable { velocity: Vec2 }
#[derive(Component)] struct Health { current: i32 }
#[derive(Component)] struct EnemyFireTimer(Timer);
#[derive(Component)] struct Explosion { timer: Timer }
#[derive(Component)] struct FloatingText { timer: Timer }

// UI Markers
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct LivesText;
#[derive(Component)] struct MainMessage;

// 🌊 ÉTATS DES VAGUES
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting }

#[derive(Resource)]
struct WaveManager {
    current_level: u32,
    current_wave: u32,
    state: WaveState,
    enemies_spawned: usize,
    spawn_timer: Timer,
    wave_timer: Timer,
    enemies_killed: usize, // Ceux que tu as vraiment détruits
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_level: 1, current_wave: 1, state: WaveState::Spawning,
            enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(3.0, TimerMode::Once),
            enemies_killed: 0,
        }
    }
}

#[derive(Resource, Default)]
struct GameState {
    score: u32,
    game_over: bool,
    exit_timer: Option<Timer>,
}

// 🎬 SYSTÈMES

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    
    commands.spawn((
        Player, Movable { velocity: Vec2::ZERO }, Health { current: PLAYER_HEALTH },
        Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() },
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));

    commands.spawn(Node { width: Val::Percent(100.0), height: Val::Px(50.0), justify_content: JustifyContent::SpaceBetween, ..default() })
        .with_children(|parent| {
            parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() }));
            parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() }));
            parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() }));
        });

    commands.spawn((MainMessage, Text::new(""), TextFont { font_size: 40.0, ..default() }, Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() }));
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) {
    if let Ok(mut movable) = query.single_mut() {
        let mut dir = 0.0;
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; }
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; }
        movable.velocity.x = dir * PLAYER_SPEED;
    }
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) {
    if kb.just_pressed(KeyCode::Space) {
        if let Ok(transform) = query.single() {
            commands.spawn((
                Bullet { from_player: true }, Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)),
            ));
        }
    }
}

fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>, player_q: Query<&Transform, With<Player>>, mut enemy_q: Query<(&Transform, &mut EnemyFireTimer)>) {
    let player_pos = player_q.single().map(|t| t.translation.xy()).ok();
    let window = window_q.single().expect("Pas de fenêtre");
    let (w, h) = (window.width()/2.0, window.height()/2.0);

    for (e_trans, mut timer) in enemy_q.iter_mut() {
        timer.0.tick(time.delta());
        let e_pos = e_trans.translation.xy();
        if timer.0.just_finished() && e_pos.x.abs() < w && e_pos.y.abs() < h {
            let mut dir = Vec2::new(0.0, -1.0);
            if let Some(p_pos) = player_pos { dir = (p_pos - e_pos).normalize(); }
            commands.spawn((
                Bullet { from_player: false }, Movable { velocity: dir * (ENEMY_SPEED * 1.5) },
                Sprite { image: asset_server.load("sprites/bullet_02.png"), custom_size: Some(BULLET_SIZE), color: Color::srgb(1.0, 0.2, 0.2), ..default() },
                Transform::from_translation(e_trans.translation),
            ));
        }
    }
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().expect("Pas de fenêtre");
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;

    for (entity, movable, mut trans) in query.iter_mut() {
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs();
        if trans.translation.y < -200.0 && trans.translation.y > -400.0 {
            trans.translation.x = trans.translation.x.clamp(-half_w + 20.0, half_w - 20.0);
        }
        // IMPORTANT : Si un alien sort en bas, on le détruit pour libérer la vague suivante
        if trans.translation.y < -half_h - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) {
    let window = window_q.single().expect("Pas de fenêtre");
    
    match wave_mgr.state {
        WaveState::Spawning => {
            wave_mgr.spawn_timer.tick(time.delta());
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 {
                let is_boss = wave_mgr.enemies_spawned == 9;
                let (kind, size, sprite) = if is_boss { (EnemyType::Boss, ENEMY_SIZE * 2.5, "sprites/alien_red.png") } else { (EnemyType::Soldier, ENEMY_SIZE, "sprites/alien_grey.png") };
                let start_x = (rand::random::<f32>() - 0.5) * window.width() * 0.8;
                
                commands.spawn((
                    Enemy { kind }, Movable { velocity: Vec2::new(0.0, -ENEMY_SPEED) },
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)),
                    Sprite { image: asset_server.load(sprite), custom_size: Some(size), ..default() },
                    Transform::from_xyz(start_x, window.height()/2.0 + 20.0, 0.0),
                ));
                wave_mgr.enemies_spawned += 1;
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; }
            }
        },
        WaveState::Fighting => {
            // SI PLUS D'ALIENS À L'ÉCRAN (morts ou sortis), ON PASSE À LA SUITE
            if enemy_q.iter().count() == 0 {
                wave_mgr.state = WaveState::Waiting;
                wave_mgr.wave_timer.reset();
            }
        },
        WaveState::Waiting => {
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() {
                wave_mgr.current_wave += 1;
                if wave_mgr.current_wave > 5 { wave_mgr.current_level += 1; wave_mgr.current_wave = 1; }
                wave_mgr.enemies_spawned = 0;
                wave_mgr.enemies_killed = 0;
                wave_mgr.state = WaveState::Spawning;
            }
        }
    }
}

fn collision_system(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform, &Enemy)>, mut player_q: Query<(&Transform, &mut Health), With<Player>>) {
    if let Ok((p_trans, mut p_health)) = player_q.single_mut() {
        let p_pos = p_trans.translation.xy();
        for (e_ent, e_trans, e_info) in enemy_q.iter() {
            let e_pos = e_trans.translation.xy();
            if p_pos.distance(e_pos) < 25.0 {
                commands.entity(e_ent).despawn();
                p_health.current -= 1;
                if p_health.current <= 0 { state.game_over = true; }
            }
            for (b_ent, b_trans, b_type) in bullet_q.iter() {
                if b_type.from_player && b_trans.translation.xy().distance(e_pos) < 25.0 {
                    let pts = match e_info.kind { EnemyType::Boss => 50, EnemyType::Soldier => 10 };
                    state.score += pts;
                    wave_mgr.enemies_killed += 1;
                    
                    commands.spawn((Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() }, Transform::from_translation(e_trans.translation)));
                    commands.spawn((FloatingText { timer: Timer::from_seconds(1.0, TimerMode::Once) }, Text2d::new(format!("+{}", pts)), TextFont { font_size: 30.0, ..default() }, TextColor(if pts == 50 { Color::srgb(1.0, 1.0, 0.0) } else { Color::WHITE }), Transform::from_translation(e_trans.translation + Vec3::new(0.0, 20.0, 2.0))));

                    commands.entity(e_ent).despawn();
                    commands.entity(b_ent).despawn();
                }
            }
        }
    }
}

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosions: Query<(Entity, &mut Explosion)>, mut floating: Query<(Entity, &mut FloatingText, &mut Transform)>) {
    for (e, mut ex) in explosions.iter_mut() {
        ex.timer.tick(time.delta());
        if ex.timer.is_finished() { commands.entity(e).despawn(); }
    }
    for (e, mut ft, mut tr) in floating.iter_mut() {
        ft.timer.tick(time.delta());
        tr.translation.y += 1.5;
        if ft.timer.is_finished() { commands.entity(e).despawn(); }
    }
}

fn ui_system(wave_mgr: Res<WaveManager>, state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) {
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} Lvl: {}", wave_mgr.current_wave, wave_mgr.current_level); }
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", state.score); }
    if let Ok(hp) = player_q.single() {
        for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp.current); }
    }

    for mut t in texts.p3().iter_mut() {
        if state.game_over { **t = "GAME OVER".to_string(); }
        else if wave_mgr.state == WaveState::Waiting {
            if wave_mgr.enemies_killed == 10 { **t = "PERFECT !!!".to_string(); }
            else { **t = "VAGUE TERMINÉE".to_string(); }
        } else { **t = "".to_string(); }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1)))
        .init_resource::<GameState>()
        .init_resource::<WaveManager>()
        .add_systems(Startup, setup_game)
        .add_systems(Update, (
            player_control_system, player_shoot_system, enemy_shoot_system,
            movement_system, wave_system, collision_system, cleanup_system, ui_system
        ))
        .run();
}