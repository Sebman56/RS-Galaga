// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-12-28 à 14h29
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::app::AppExit;

// 🎮 CONSTANTES
const PLAYER_SPEED: f32 = 500.0;
const BULLET_SPEED: f32 = 700.0;
const ENEMY_SPEED: f32 = 120.0;
const PLAYER_SIZE: Vec2 = Vec2::new(35.0, 20.0);
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0);
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0);
const PLAYER_HEALTH: i32 = 3;

// 📦 COMPOSANTS
#[derive(Component)] struct Player;
#[derive(Component)] struct Enemy;
#[derive(Component)] struct Bullet { from_player: bool }
#[derive(Component)] struct Movable { velocity: Vec2 }
#[derive(Component)] struct Health { current: i32 }
#[derive(Component)] struct EnemyFireTimer(Timer);

// UI Markers
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct LevelText;
#[derive(Component)] struct LivesText;
#[derive(Component)] struct MainMessage;

// 🌊 ÉNUMÉRATIONS
#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right }
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting }

// 🗃️ RESSOURCES
#[derive(Resource)]
struct WaveManager {
    current_level: u32,
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
            current_level: 1,
            current_wave: 1,
            state: WaveState::Spawning,
            direction: SpawnDirection::Top,
            enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            wave_timer: Timer::from_seconds(4.0, TimerMode::Once),
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

// 🎬 SYSTÈMES

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    // Joueur
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

    // UI Bandeau
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        padding: UiRect::horizontal(Val::Px(20.0)),
        ..default()
    }).with_children(|parent| {
        parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() }));
        parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() }));
        parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() }));
    });

    // Message Central
    commands.spawn((
        MainMessage,
        Text::new(""),
        TextFont { font_size: 40.0, ..default() },
        Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() },
    ));
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) {
    if let Ok(mut movable) = query.single_mut() {
        let mut dir = 0.0;
        if kb.pressed(KeyCode::ArrowLeft) || kb.pressed(KeyCode::KeyA) { dir -= 1.0; }
        if kb.pressed(KeyCode::ArrowRight) || kb.pressed(KeyCode::KeyD) { dir += 1.0; }
        movable.velocity.x = dir * PLAYER_SPEED;
    }
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) {
    if kb.just_pressed(KeyCode::Space) {
        if let Ok(transform) = query.single() {
            commands.spawn((
                Bullet { from_player: true },
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) },
                Sprite { image: asset_server.load("sprites/laser_blue.png"), custom_size: Some(BULLET_SIZE), ..default() },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)),
            ));
        }
    }
}

fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut query: Query<(&Transform, &mut EnemyFireTimer), With<Enemy>>) {
    for (transform, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.spawn((
                Bullet { from_player: false },
                Movable { velocity: Vec2::new(0.0, -BULLET_SPEED * 0.4) },
                Sprite {
                    image: asset_server.load("sprites/laser_red.png"),
                    custom_size: Some(BULLET_SIZE),
                    color: Color::srgb(1.0, 0.0, 0.0),
                    ..default()
                },
                Transform::from_translation(transform.translation - Vec3::new(0.0, 20.0, 0.0)),
            ));
        }
    }
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().expect("Window error");
    let half_w = window.width() / 2.0;
    let half_h = window.height() / 2.0;
    for (entity, movable, mut transform) in query.iter_mut() {
        transform.translation += movable.velocity.extend(0.0) * time.delta_secs();
        if transform.translation.y < -200.0 && transform.translation.y > -400.0 {
            transform.translation.x = transform.translation.x.clamp(-half_w + 20.0, half_w - 20.0);
        }
        if transform.translation.y.abs() > half_h + 100.0 || transform.translation.x.abs() > half_w + 100.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, mut game_state: ResMut<GameState>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) {
    let window = window_q.single().expect("Window error");
    let enemy_count = enemy_q.iter().count();

    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) {
        (1, 3) => SpawnDirection::Right, (1, 4) => SpawnDirection::Left,
        (2, 1) | (2, 3) | (3, 1) | (3, 5) => SpawnDirection::Right,
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left,
        (3, 3) => SpawnDirection::Top, _ => SpawnDirection::Top,
    };

    match wave_mgr.state {
        WaveState::Spawning => {
            wave_mgr.spawn_timer.tick(time.delta());
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 {
                let (start_pos, velocity) = match wave_mgr.direction {
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)),
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)),
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)),
                };
                commands.spawn((Enemy, Movable { velocity }, EnemyFireTimer(Timer::from_seconds(2.0, TimerMode::Repeating)), 
                    Sprite { image: asset_server.load("sprites/enemy_01.png"), custom_size: Some(ENEMY_SIZE), ..default() },
                    Transform::from_translation(start_pos)));
                wave_mgr.enemies_spawned += 1;
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; }
            }
        },
        WaveState::Fighting => if enemy_count == 0 {
            if wave_mgr.current_wave >= 5 {
                if wave_mgr.current_level >= 3 { game_state.victory = true; } 
                else { wave_mgr.current_level += 1; wave_mgr.current_wave = 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); }
            } else { wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); }
        },
        WaveState::Waiting => {
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() && !game_state.victory {
                wave_mgr.enemies_spawned = 0; wave_mgr.state = WaveState::Spawning;
            }
        }
    }
}

fn collision_system(mut commands: Commands, mut state: ResMut<GameState>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform), With<Enemy>>, mut player_q: Query<(Entity, &Transform, &mut Health), With<Player>>) {
    for (b_ent, b_trans, b_type) in bullet_q.iter() {
        if b_type.from_player {
            for (e_ent, e_trans) in enemy_q.iter() {
                if b_trans.translation.distance(e_trans.translation) < 25.0 {
                    commands.entity(e_ent).despawn(); commands.entity(b_ent).despawn(); state.score += 10;
                }
            }
        } else if let Ok((p_ent, p_trans, mut p_health)) = player_q.single_mut() {
            if b_trans.translation.distance(p_trans.translation) < 20.0 {
                commands.entity(b_ent).despawn(); p_health.current -= 1;
                if p_health.current <= 0 { commands.entity(p_ent).despawn(); state.game_over = true; }
            }
        }
    }
}

fn ui_system(wave_mgr: Res<WaveManager>, game_state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) {
    let dir = match wave_mgr.direction { SpawnDirection::Top => "Haut", SpawnDirection::Left => "Gauche", SpawnDirection::Right => "Droite" };
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} ({}) Lvl: {}", wave_mgr.current_wave, dir, wave_mgr.current_level); }
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", game_state.score); }
    let hp = player_q.single().map(|h| h.current).unwrap_or(0);
    for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp); }
    for mut t in texts.p3().iter_mut() {
        if game_state.victory { **t = format!("VICTOIRE ! Score: {}\nR: Rejouer | Q: Quitter", game_state.score); }
        else if game_state.game_over { **t = "GAME OVER\nLes aliens continuent...".to_string(); }
        else if wave_mgr.state == WaveState::Waiting && wave_mgr.current_wave == 1 { **t = format!("LEVEL {}", wave_mgr.current_level); }
        else { **t = "".to_string(); }
    }
}

fn input_game_system(
    mut commands: Commands, 
    kb: Res<ButtonInput<KeyCode>>, 
    mut game_state: ResMut<GameState>, 
    mut wave_mgr: ResMut<WaveManager>, 
    mut exit: MessageWriter<AppExit>, // Correction MessageWriter 0.17
    time: Res<Time>, 
    all_ents: Query<Entity, Or<(With<Player>, With<Enemy>, With<Bullet>)>>, 
    asset_server: Res<AssetServer>
) {
    if (game_state.victory || game_state.game_over) && kb.just_pressed(KeyCode::KeyR) {
        for e in all_ents.iter() { commands.entity(e).despawn(); }
        *game_state = GameState::default(); *wave_mgr = WaveManager::default();
        commands.spawn((Player, Movable { velocity: Vec2::ZERO }, Health { current: PLAYER_HEALTH }, 
            Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() },
            Transform::from_xyz(0.0, -300.0, 0.0)));
    }

    if (game_state.victory || game_state.game_over) && kb.just_pressed(KeyCode::KeyQ) {
        game_state.exit_timer = Some(Timer::from_seconds(1.0, TimerMode::Once));
        commands.spawn((Text::new("Au revoir."), TextFont { font_size: 60.0, ..default() }, TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() }));
    }

    if let Some(ref mut timer) = game_state.exit_timer {
        timer.tick(time.delta());
        if timer.just_finished() {
            exit.write(AppExit::Success); // Syntaxe correcte 0.17 : .write()
        }
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
            movement_system, wave_system, collision_system, ui_system, input_game_system,
        ))
        .run();
}