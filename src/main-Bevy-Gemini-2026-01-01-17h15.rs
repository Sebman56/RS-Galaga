// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-01 à 17h15
// ═══════════════════════════════════════════════════════════════════════════
//
//
// ═══════════════════════════════════════════════════════════════════════════
// 🚀 XGALAGA : VERSION ULTIME AVEC COMMENTAIRES LIGNE PAR LIGNE
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*; // Importation de la bibliothèque Bevy pour créer le jeu
use bevy::window::PrimaryWindow; // Outil pour obtenir les infos sur la fenêtre
use bevy::app::AppExit; // Outil pour fermer le programme proprement
use bevy::math::Vec3Swizzles; // Raccourci pour manipuler les vecteurs (X, Y)

const PLAYER_SPEED: f32 = 500.0; // Vitesse de déplacement de ton vaisseau
const BULLET_SPEED: f32 = 700.0; // Vitesse de tes tirs laser
const ENEMY_SPEED: f32 = 120.0; // Vitesse de base des aliens
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0); // Taille physique de ton vaisseau
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0); // Taille physique d'un alien soldat
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0); // Taille d'un projectile laser
const PLAYER_HEALTH: i32 = 3; // Ton nombre de vies au lancement du jeu

#[derive(Component)] struct Player; // Étiquette pour identifier ton vaisseau
#[derive(Component)] struct Enemy { kind: EnemyType } // Étiquette pour les aliens
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss } // Types d'ennemis possibles
#[derive(Component)] struct Bullet { from_player: bool } // Étiquette pour les tirs
#[derive(Component)] struct Movable { velocity: Vec2 } // Composant pour les objets qui bougent
#[derive(Component)] struct Health { current: i32 } // Composant pour gérer les points de vie
#[derive(Component)] struct EnemyFireTimer(Timer); // Chronomètre de tir pour chaque alien
#[derive(Component)] struct Explosion { timer: Timer } // Durée de vie de l'image d'explosion
#[derive(Component)] struct FloatingText { timer: Timer } // Durée de vie du score qui monte

#[derive(Component)] struct ScoreText; // Marqueur pour le texte du score
#[derive(Component)] struct LevelText; // Marqueur pour le texte du niveau
#[derive(Component)] struct LivesText; // Marqueur pour le texte des vies
#[derive(Component)] struct MainMessage; // Marqueur pour les messages au centre

#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right } // Origines possibles
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting } // États de vague

#[derive(Resource)] struct WaveManager { // Structure qui gère tout le cycle du jeu
    current_level: u32, // Niveau actuel (1, 2, 3...)
    current_wave: u32, // Vague actuelle (1 à 5)
    state: WaveState, // Phase actuelle (Création, Combat, Pause)
    direction: SpawnDirection, // Direction d'arrivée choisie
    enemies_spawned: usize, // Nombre d'aliens déjà créés dans la vague
    spawn_timer: Timer, // Cadence d'apparition des aliens
    wave_timer: Timer, // Durée de la pause entre deux vagues
    enemies_killed: usize, // Nombre d'aliens abattus par le joueur
}

impl Default for WaveManager { // Valeurs par défaut au lancement
    fn default() -> Self { // Initialisation de la ressource
        Self { // Création de l'objet WaveManager
            current_level: 1, // On commence au niveau 1
            current_wave: 1, // On commence à la vague 1
            state: WaveState::Spawning, // On commence par créer des aliens
            direction: SpawnDirection::Top, // Direction par défaut : le haut
            enemies_spawned: 0, // Aucun alien créé pour l'instant
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating), // Un alien toutes les 0.6s
            wave_timer: Timer::from_seconds(2.5, TimerMode::Once), // 2.5s de pause entre vagues
            enemies_killed: 0, // Zéro trophée au compteur
        } // Fin de l'objet
    } // Fin de la fonction
}

#[derive(Resource, Default)] struct GameState { // Ressource pour l'état général
    score: u32, // Ton score cumulé
    game_over: bool, // Vrai si tu as perdu
    victory: bool, // Vrai si tu as fini le jeu
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) { // Préparation initiale
    commands.spawn(Camera2d); // On installe la caméra pour voir le jeu
    commands.spawn(( // On crée le joueur
        Player, // On lui met l'étiquette Player
        Movable { velocity: Vec2::ZERO }, // Il commence sans bouger
        Health { current: PLAYER_HEALTH }, // On lui donne ses 3 vies
        Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() }, // Son image
        Transform::from_xyz(0.0, -300.0, 0.0), // Sa position de départ en bas
    )); // Fin de la création du joueur
    commands.spawn(Node { width: Val::Percent(100.0), height: Val::Px(50.0), justify_content: JustifyContent::SpaceBetween, ..default() }) // Barre du haut
        .with_children(|parent| { // On ajoute les textes dedans
            parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() })); // Texte niveau
            parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() })); // Texte score
            parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() })); // Texte vies
        }); // Fin de l'UI
    commands.spawn((MainMessage, Text::new(""), TextFont { font_size: 40.0, ..default() }, Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() })); // Message central
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) { // Système de mouvement
    if let Ok(mut movable) = query.get_single_mut() { // Si le joueur existe
        let mut dir = 0.0; // Direction horizontale par défaut
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; } // Gauche si touche pressée
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; } // Droite si touche pressée
        movable.velocity.x = dir * PLAYER_SPEED; // Application de la vitesse
    } // Fin du bloc joueur
}

fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) { // Système de tir
    if kb.just_pressed(KeyCode::Space) { // Si touche Espace juste appuyée
        if let Ok(transform) = query.get_single() { // Si le joueur est vivant
            commands.spawn(( // On crée une balle
                Bullet { from_player: true }, // C'est une balle du joueur
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) }, // Elle monte vite
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() }, // Son image
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)), // Départ devant le vaisseau
            )); // Fin de la balle
        } // Fin du bloc joueur
    } // Fin du bloc Espace
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_q: Query<&Window, With<PrimaryWindow>>) { // Système de déplacement global
    let window = window_q.get_single().expect("Fenêtre absente"); // Récupération de la fenêtre
    let (hw, hh) = (window.width()/2.0, window.height()/2.0); // Calcul des demi-dimensions
    for (entity, movable, mut trans) in query.iter_mut() { // Pour chaque objet qui bouge
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs(); // On change sa position
        if trans.translation.y < -200.0 { trans.translation.x = trans.translation.x.clamp(-hw + 20.0, hw - 20.0); } // Limites pour le joueur
        if trans.translation.y.abs() > hh + 100.0 || trans.translation.x.abs() > hw + 100.0 { commands.entity(entity).despawn(); } // Destruction si hors-écran
    } // Fin de la boucle
}

fn wave_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut wave_mgr: ResMut<WaveManager>, mut game_state: ResMut<GameState>, enemy_q: Query<&Enemy>, window_q: Query<&Window, With<PrimaryWindow>>) { // Système des vagues
    let window = window_q.get_single().expect("Window error"); // Vérification de la fenêtre
    let enemy_count = enemy_q.iter().count(); // Comptage des aliens restants
    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) { // Sélection de la direction
        (1, 3) => SpawnDirection::Right, // Niveau 1 Vague 3 -> Droite
        (1, 4) => SpawnDirection::Left, // Niveau 1 Vague 4 -> Gauche
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right, // Autres cas Droite
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left, // Autres cas Gauche
        _ => SpawnDirection::Top, // Par défaut -> Haut
    }; // Fin du choix direction
    match wave_mgr.state { // Analyse de l'état actuel
        WaveState::Spawning => { // Si on crée des aliens
            wave_mgr.spawn_timer.tick(time.delta()); // On fait avancer le chrono d'apparition
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 { // Si c'est le moment d'en créer un
                let is_boss = wave_mgr.enemies_spawned == 9; // Le 10ème est un Boss
                let sprite_path = if is_boss { "sprites/alien_red.png" } else { match wave_mgr.direction { // Choix image
                    SpawnDirection::Left => "sprites/alien_red.png", // Rouge depuis la gauche
                    SpawnDirection::Right => "sprites/alien_green.png", // Vert depuis la droite
                    SpawnDirection::Top => "sprites/alien_grey.png", // Gris depuis le haut
                }}; // Fin choix image
                let (start_pos, velocity) = match wave_mgr.direction { // Position et vitesse
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)), // Arrivée du haut
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)), // Arrivée de gauche
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)), // Arrivée de droite
                }; // Fin calcul position
                commands.spawn(( // Création de l'alien
                    Enemy { kind: if is_boss { EnemyType::Boss } else { EnemyType::Soldier } }, // Soldat ou Boss
                    Movable { velocity }, // Sa vitesse calculée
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)), // Cadence de tir
                    Sprite { image: asset_server.load(sprite_path), custom_size: Some(if is_boss { ENEMY_SIZE * 2.5 } else { ENEMY_SIZE }), ..default() }, // Apparence
                    Transform::from_translation(start_pos) // Position de départ
                )); // Fin création
                wave_mgr.enemies_spawned += 1; // Un alien de plus au compteur
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; } // Fin d'apparition après 10
            } // Fin chrono fini
        }, // Fin état Spawning
        WaveState::Fighting => if enemy_count == 0 { // Si tous les ennemis sont éliminés
            if wave_mgr.current_wave >= 5 { // Si vague 5 terminée
                if wave_mgr.current_level >= 3 { game_state.victory = true; } // Victoire finale au niveau 3
                else { wave_mgr.current_level += 1; wave_mgr.current_wave = 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } // Niveau suivant
            } else { wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); } // Vague suivante
        }, // Fin état Fighting
        WaveState::Waiting => { // Phase de pause
            wave_mgr.wave_timer.tick(time.delta()); // Avancement du chrono pause
            if wave_mgr.wave_timer.is_finished() && !game_state.victory { // Si pause finie
                wave_mgr.enemies_spawned = 0; // Reset compteur création
                wave_mgr.enemies_killed = 0; // Reset compteur destruction
                wave_mgr.state = WaveState::Spawning; // On relance la création
            } // Fin pause finie
        } // Fin état Waiting
    } // Fin match state
} // Fin système wave

fn collision_system(mut commands: Commands, asset_server: Res<AssetServer>, mut state: ResMut<GameState>, mut wave_mgr: ResMut<WaveManager>, bullet_q: Query<(Entity, &Transform, &Bullet)>, enemy_q: Query<(Entity, &Transform, &Enemy)>, mut player_q: Query<(&Transform, &mut Health), With<Player>>) { // Système de chocs
    if let Ok((p_trans, mut p_health)) = player_q.get_single_mut() { // Si le joueur est là
        for (e_ent, e_trans, e_info) in enemy_q.iter() { // Pour chaque alien
            let e_pos = e_trans.translation.xy(); // Position de l'alien
            if p_trans.translation.xy().distance(e_pos) < 25.0 { // Si le joueur touche l'alien
                commands.entity(e_ent).despawn(); // L'alien explose
                p_health.current -= 1; // Le joueur perd 1 vie
                if p_health.current <= 0 { state.game_over = true; } // Game over si plus de vie
            } // Fin choc joueur-alien
            for (b_ent, b_trans, b_type) in bullet_q.iter() { // Pour chaque balle
                if b_type.from_player && b_trans.translation.xy().distance(e_pos) < 25.0 { // Si ta balle touche l'alien
                    let pts = match e_info.kind { EnemyType::Boss => 50, EnemyType::Soldier => 10 }; // Points selon type
                    state.score += pts; // Ajout au score total
                    wave_mgr.enemies_killed += 1; // Un mort de plus pour la statistique
                    commands.spawn((Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) }, Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() }, Transform::from_translation(e_trans.translation))); // Effet BOUM
                    commands.spawn((FloatingText { timer: Timer::from_seconds(1.0, TimerMode::Once) }, Text2d::new(format!("+{}", pts)), TextFont { font_size: 30.0, ..default() }, TextColor(if pts == 50 { Color::srgb(1.0, 1.0, 0.0) } else { Color::WHITE }), Transform::from_translation(e_trans.translation + Vec3::new(20.0, 10.0, 2.0)))); // Score flottant
                    commands.entity(e_ent).despawn(); // Destruction alien
                    commands.entity(b_ent).despawn(); // Destruction balle
                } // Fin impact
            } // Fin boucle balles
        } // Fin boucle aliens
    } // Fin bloc joueur
} // Fin système collisions

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosions: Query<(Entity, &mut Explosion)>, mut floating: Query<(Entity, &mut FloatingText, &mut Transform)>) { // Nettoyage des effets
    for (e, mut ex) in explosions.iter_mut() { // Pour chaque explosion
        ex.timer.tick(time.delta()); // Avancement chrono
        if ex.timer.is_finished() { commands.entity(e).despawn(); } // Disparition si fini
    } // Fin boucle explosions
    for (e, mut ft, mut tr) in floating.iter_mut() { // Pour chaque score flottant
        ft.timer.tick(time.delta()); // Avancement chrono
        tr.translation.y += 1.5; // Le texte monte vers le haut
        if ft.timer.is_finished() { commands.entity(e).despawn(); } // Disparition si fini
    } // Fin boucle scores
} // Fin système cleanup

fn ui_system(wave_mgr: Res<WaveManager>, state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) { // Système d'affichage
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} Lvl: {}", wave_mgr.current_wave, wave_mgr.current_level); } // Mise à jour texte vague
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", state.score); } // Mise à jour texte score
    if let Ok(hp) = player_q.get_single() { for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp.current); } } // Mise à jour texte vies
    for mut t in texts.p3().iter_mut() { // Gestion des messages centraux
        if state.game_over { **t = "GAME OVER".to_string(); } // Affichage mort
        else if state.victory { **t = "VICTOIRE TOTALE !".to_string(); } // Affichage victoire
        else if wave_mgr.state == WaveState::Waiting { **t = "VAGUE TERMINÉE".to_string(); } // Affichage fin vague
        else { **t = "".to_string(); } // Pas de message sinon
    } // Fin boucle message
} // Fin système UI

fn main() { // Point d'entrée du programme
    App::new() // Création de l'application Bevy
        .add_plugins(DefaultPlugins) // Ajout des plugins standards (fenêtre, sons, images)
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.1))) // Couleur de fond (Espace)
        .init_resource::<GameState>() // Création de la ressource score
        .init_resource::<WaveManager>() // Création de la ressource vagues
        .add_systems(Startup, setup_game) // Lancement de l'initialisation au démarrage
        .add_systems(Update, (player_control_system, player_shoot_system, movement_system, wave_system, collision_system, cleanup_system, ui_system)) // Boucle de jeu
        .run(); // Allumage du moteur de jeu
} // Fin du programme