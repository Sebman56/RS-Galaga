
// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-01-02 à 02h13
// ═══════════════════════════════════════════════════════════════════════════
//
//
// Message correct
// le vaisseau ok
// Utilisation des touches "Q" "X" pour sortir, "P" pour pause, "R" pour recommencer le jeu
// Aliens du haut de couelur gris, de gauvhe de couelur gauche, de droite de couelur verte
// Forcage des 3 premieres vagues, 1er Level: 
//          La 1ere vague vient de la la gauche, 
//          la 2eme vague vient de la droite et 
//          la 3eme vague vient du haut
//
//
//
// Attribuer un sprite pour chaque tir bonus
//
// ═══════════════════════════════════════════════════════════════════════════
// 🛸 XGALAGA RUST - VERSION EXPLIQUÉE POUR LES FUTURS GÉNIES
// ═══════════════════════════════════════════════════════════════════════════

use bevy::prelude::*; // On importe les outils de Bevy pour fabriquer le jeu.
use bevy::window::PrimaryWindow; // On importe l'outil pour regarder la fenêtre du jeu.
use bevy::app::AppExit; // On importe l'outil pour pouvoir fermer le jeu proprement.

const PLAYER_SPEED: f32 = 500.0; // La vitesse de notre vaisseau (il va vite !).
const BULLET_SPEED: f32 = 700.0; // La vitesse des balles qui filent dans l'espace.
const ENEMY_SPEED: f32 = 120.0; // La vitesse des méchants aliens qui descendent.
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0); // La taille du vaisseau du joueur.
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0); // La taille des petits aliens.
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0); // La taille des projectiles.
const PLAYER_HEALTH: i32 = 3; // Le nombre de vies (3 coeurs pour commencer).

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)] // On prépare les outils pour les états.
enum AppState { #[default] Running, Paused } // Le jeu peut être soit "En marche", soit "En pause".

#[derive(Component)] struct Player; // Une étiquette pour dire : "Ça, c'est le joueur !".
#[derive(Component)] struct Enemy { kind: EnemyType } // Une étiquette pour dire : "Ça, c'est un méchant !".
#[derive(Component, PartialEq)] enum EnemyType { Soldier, Boss } // Il y a des petits soldats et des gros chefs.
#[derive(Component)] struct Bullet { from_player: bool } // Une étiquette pour savoir qui a tiré la balle.
#[derive(Component)] struct Movable { velocity: Vec2 } // Une étiquette pour les objets qui peuvent bouger.
#[derive(Component)] struct Health { current: i32 } // Une étiquette pour compter les points de vie.
#[derive(Component)] struct EnemyFireTimer(Timer); // Un petit chrono pour que l'alien tire régulièrement.
#[derive(Component)] struct Explosion { timer: Timer } // Un chrono pour que l'explosion disparaisse vite.
#[derive(Component)] struct FloatingScore { timer: Timer } // Un chrono pour le petit texte "+10" qui monte.

#[derive(Component)] struct ScoreText; // Étiquette pour le texte du score en haut.
#[derive(Component)] struct LevelText; // Étiquette pour afficher le niveau actuel.
#[derive(Component)] struct LivesText; // Étiquette pour afficher nos vies restantes.
#[derive(Component)] struct MainMessage; // Étiquette pour les gros messages au milieu de l'écran.

#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right } // D'où viennent les aliens ?
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, LevelCompleted, Waiting } // Que font les aliens ?

#[derive(Resource)] // Une ressource, c'est comme une boîte à outils partagée.
struct WaveManager { // On crée le gestionnaire des vagues d'ennemis.
    current_level: u32, // Le numéro du niveau (1, 2, 3...).
    current_wave: u32, // Le numéro de la vague dans le niveau.
    state: WaveState, // L'état actuel de la vague (arrivée, combat...).
    direction: SpawnDirection, // La direction choisie pour l'attaque.
    enemies_spawned: usize, // Combien d'aliens sont déjà apparus.
    enemies_killed_by_player: usize, // Combien d'aliens tu as détruits toi-même.
    spawn_timer: Timer, // Le chrono entre chaque apparition d'alien.
    wave_timer: Timer, // Le chrono de repos entre deux vagues.
    show_good_job: bool, // Est-ce qu'on doit afficher "Bravo" ?
}

impl Default for WaveManager { // On définit les réglages de départ de la boîte à outils.
    fn default() -> Self { // C'est ici que tout commence à zéro.
        Self {
            current_level: 1, // On commence au niveau 1.
            current_wave: 1, // On commence à la vague 1.
            state: WaveState::Spawning, // On commence par faire apparaître les aliens.
            direction: SpawnDirection::Top, // Ils arrivent par le haut au début.
            enemies_spawned: 0, // Personne n'est encore apparu.
            enemies_killed_by_player: 0, // Tu n'as encore tué personne.
            spawn_timer: Timer::from_seconds(0.6, TimerMode::Repeating), // Un alien toutes les 0.6 secondes.
            wave_timer: Timer::from_seconds(2.0, TimerMode::Once), // 2 secondes de pause.
            show_good_job: false, // On n'affiche pas encore le bravo.
        }
    }
}

#[derive(Resource, Default)] // Une autre ressource pour l'état général du jeu.
struct GameState { // Le "cerveau" du jeu.
    score: u32, // Ton score total.
    game_over: bool, // Est-ce que tu as perdu ?
    victory: bool, // Est-ce que tu as gagné ?
}


#[derive(Clone, Copy, PartialEq)]
enum WeaponMode {
    Single, DoubleJumelé, DoubleV, Triple, Quadruple, Quintuple, Sixtuple, Septuple,
    Rapid2, Rapid3, Rapid4, Rapid5 // Balles l'une après l'autre
}

#[derive(Component)]
struct PlayerStats {
    weapon: WeaponMode,
    rapid_fire_timer: Timer,
    bullets_left_to_fire: u32,
}

#[derive(Component)]
enum BonusType {
    Weapon(WeaponMode),
    ExtraLife,
    NextLevel,
}

#[derive(Component)]
struct PowerUp {
    kind: BonusType,
}

fn main() { // La fonction principale : c'est le bouton "START" du code.
    App::new() // On crée une nouvelle application de jeu.
        .add_plugins(DefaultPlugins) // On installe tous les outils de base (sons, images, fenêtre).
        .insert_resource(ClearColor(Color::BLACK)) // On peint le fond de l'espace en noir.
        .init_resource::<GameState>() // On prépare le cerveau du jeu.
        .init_resource::<WaveManager>() // On prépare le chef des aliens.
        .init_state::<AppState>() // On active le système de pause/marche.
        .add_systems(Startup, setup_game) // On lance le système de départ une seule fois.
        .add_systems(Update, (input_system, ui_update_system)) // On surveille le clavier et les textes tout le temps.
        .add_systems(Update, ( // On lance tous ces systèmes seulement quand le jeu tourne.
            player_control_system, player_shoot_system, // Bouger et tirer.
            enemy_shoot_system, movement_system, wave_system, // Les aliens bougent, tirent et arrivent.
            collision_system, cleanup_system // On gère les chocs et le nettoyage des objets.
        ).run_if(in_state(AppState::Running))) // Tout ça s'arrête si on fait pause.
        .run(); // On allume le moteur du jeu !
}

fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) { // La mise en place du décor.
    commands.spawn(Camera2d); // On pose une caméra pour voir ce qui se passe.
    spawn_player(&mut commands, &asset_server); // On fait apparaître ton vaisseau.

    commands.spawn(Node { // On crée une zone invisible en haut pour le texte.
        width: Val::Percent(100.0), // Ça prend toute la largeur.
        height: Val::Px(50.0), // Ça fait 50 pixels de haut.
        justify_content: JustifyContent::SpaceBetween, // On écarte les textes sur les côtés.
        padding: UiRect::all(Val::Px(15.0)), // On laisse un peu de place sur les bords.
        ..default() // On remplit le reste avec les réglages par défaut.
    }).with_children(|parent| { // On met des enfants (les textes) dans cette zone.
        parent.spawn((LevelText, Text::new(""), TextFont::from_font_size(20.0))); // Texte pour le niveau.
        parent.spawn((ScoreText, Text::new("Score: 0"), TextFont::from_font_size(25.0))); // Texte pour le score.
        parent.spawn((LivesText, Text::new("Vies: 3"), TextFont::from_font_size(20.0))); // Texte pour les vies.
    });

    commands.spawn(( // On crée le gros message au milieu.
        MainMessage, // On lui met l'étiquette message.
        Text::new(""), // Il est vide au début.
        TextFont::from_font_size(50.0), // C'est écrit très gros !
        Node { // On le place précisément.
            position_type: PositionType::Absolute, // On donne des coordonnées fixes.
            left: Val::Percent(35.0), // À 35% du bord gauche.
            top: Val::Percent(45.0), // À 45% du haut.
            ..default() // Le reste par défaut.
        }
    ));
}

fn spawn_player(commands: &mut Commands, asset_server: &Res<AssetServer>) { // Fabriquer le vaisseau.
    commands.spawn(( // On crée l'entité du joueur.
        Player, // On lui met l'étiquette Joueur.
        Movable { velocity: Vec2::ZERO }, // Il ne bouge pas encore.
        Health { current: PLAYER_HEALTH }, // On lui donne ses 3 coeurs.
        PlayerStats { 
            weapon: WeaponMode::Single, 
            rapid_fire_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            bullets_left_to_fire: 0 
            },
        Sprite { // On lui donne une image.
            image: asset_server.load("sprites/player_01.png"), // On charge l'image du vaisseau.
            custom_size: Some(PLAYER_SIZE), // On règle sa taille.
            ..default() // Le reste par défaut.
        },
        Transform::from_xyz(0.0, -300.0, 1.0), // On le pose en bas de l'écran.
    ));
}

fn input_system( // Le système qui écoute tes doigts sur le clavier.
    kb: Res<ButtonInput<KeyCode>>, // On regarde quelles touches sont appuyées.
    mut exit: MessageWriter<AppExit>, // L'outil pour fermer le jeu.
    state: Res<State<AppState>>, // On regarde si on est en pause.
    mut next_state: ResMut<NextState<AppState>>, // L'outil pour changer de mode (pause/marche).
    mut game_state: ResMut<GameState>, // L'outil pour modifier le cerveau du jeu.
    mut wave_mgr: ResMut<WaveManager>, // L'outil pour modifier les vagues d'aliens.
    mut commands: Commands, // L'outil pour donner des ordres.
    asset_server: Res<AssetServer>, // L'outil pour charger les images.
    entities_q: Query<Entity, Or<(With<Enemy>, With<Bullet>, With<Player>, With<Explosion>, With<FloatingScore>)>>, // On cherche tout le monde.
) {
    if kb.just_pressed(KeyCode::KeyQ) || kb.just_pressed(KeyCode::KeyX) || kb.just_pressed(KeyCode::Escape) { // Si tu appuies sur Q, X ou Echap...
        exit.write(AppExit::Success); // On ferme le jeu.
    }

    if kb.just_pressed(KeyCode::KeyP) { // Si tu appuies sur P...
        if *state.get() == AppState::Running { next_state.set(AppState::Paused); } // On met pause.
        else { next_state.set(AppState::Running); } // Ou on remet en marche.
    }

    if kb.just_pressed(KeyCode::KeyR) { // Si tu appuies sur R (Recommencer)...
        *game_state = GameState::default(); // On remet le cerveau à zéro.
        *wave_mgr = WaveManager::default(); // On remet les vagues à zéro.
        for entity in entities_q.iter() { // Pour chaque objet dans le jeu...
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); } // On le fait disparaître.
        }
        spawn_player(&mut commands, &asset_server); // On recrée un vaisseau tout neuf.
        next_state.set(AppState::Running); // On relance le jeu.
    }
}

fn wave_system( // Le système qui gère l'arrivée des aliens.
    mut commands: Commands, // Pour faire apparaître les méchants.
    asset_server: Res<AssetServer>, // Pour l'image de l'alien.
    time: Res<Time>, // Pour compter le temps qui passe.
    mut wave_mgr: ResMut<WaveManager>, // Pour savoir où on en est dans les niveaux.
    mut game_state: ResMut<GameState>, // Pour dire si on a gagné.
    enemy_q: Query<&Enemy>, // Pour compter combien il reste d'ennemis.
    window_q: Query<&Window, With<PrimaryWindow>> // Pour connaître la taille de l'écran.
) {
    let Ok(window) = window_q.single() else { return }; // Si on trouve la fenêtre, on continue.
    let enemy_count = enemy_q.iter().count(); // On compte les aliens sur l'écran.

    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) { // On choisit d'où ils viennent.
        
        (1, 1) => SpawnDirection::Left, // Niveau 1-1 : ils arrivent de gauche.
        (1, 2) => SpawnDirection::Right, // Niveau 1-2 : ils arrivent de droite.
        (1, 3) => SpawnDirection::Top, // Niveau 1-3 : ils arrivent de gauche.
        
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right, // D'autres niveaux de droite.
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left, // D'autres niveaux de gauche.
        _ => SpawnDirection::Top, // Sinon, ils arrivent par le haut.
    };

    match wave_mgr.state { // On regarde ce que la vague est en train de faire.
        WaveState::Spawning => { // Ils sont en train d'arriver !
            wave_mgr.spawn_timer.tick(time.delta()); // On fait avancer le chrono d'arrivée.
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 { // Si le chrono dit "Go" et qu'on est moins de 10...
                let is_boss = wave_mgr.enemies_spawned == 9; // Le 10ème alien est un Boss !
                let sprite_path = if is_boss { "sprites/alien_red.png" } else { 
                    match wave_mgr.direction {
                        SpawnDirection::Left => "sprites/alien_red.png",
                        SpawnDirection::Right => "sprites/alien_green.png",
                        SpawnDirection::Top => "sprites/alien_grey.png",
                    }
                };
                let (start_pos, velocity) = match wave_mgr.direction { // On calcule la position et la vitesse.
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)), // Arrivée par le haut.
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)), // Arrivée par la gauche.
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)), // Arrivée par la droite.
                };

                commands.spawn(( // On fabrique l'alien !
                    Enemy { kind: if is_boss { EnemyType::Boss } else { EnemyType::Soldier } }, // On définit son type.
                    Movable { velocity }, // On lui donne sa vitesse.
                    EnemyFireTimer(Timer::from_seconds(if is_boss { 1.2 } else { 2.5 }, TimerMode::Repeating)), // Son rythme de tir.
                    Sprite { image: asset_server.load(sprite_path), custom_size: Some(if is_boss { ENEMY_SIZE * 2.5 } else { ENEMY_SIZE }), ..default() }, // Son image.
                    
                    Transform::from_translation(start_pos) // On le place au point de départ.
                ));
                wave_mgr.enemies_spawned += 1; // On compte un alien de plus.
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; } // Quand il y en a 10, on passe au combat !
            }
        },
        WaveState::Fighting => { // On est en plein combat !
            if enemy_count == 0 { // Si tous les aliens sont morts...
                wave_mgr.show_good_job = wave_mgr.enemies_killed_by_player >= 10; // On vérifie si tu as bien bossé.
                if wave_mgr.current_wave >= 5 { // Si c'était la 5ème vague...
                    if wave_mgr.current_level >= 3 { game_state.victory = true; } // Si c'était le niveau 3, tu as gagné le jeu !
                    else { wave_mgr.state = WaveState::LevelCompleted; wave_mgr.wave_timer.reset(); } // Sinon, niveau suivant.
                } else { // Si c'était juste une petite vague...
                    wave_mgr.current_wave += 1; // Vague suivante.
                    wave_mgr.state = WaveState::Waiting; // Petite pause.
                    wave_mgr.wave_timer.reset(); // On remet le chrono de pause à zéro.
                }
            }
        },
        WaveState::LevelCompleted | WaveState::Waiting => { // On attend entre deux vagues.
            wave_mgr.wave_timer.tick(time.delta()); // On fait avancer le chrono de pause.
            if wave_mgr.wave_timer.is_finished() { // Si le repos est fini...
                if wave_mgr.state == WaveState::LevelCompleted { // Si on changeait de niveau...
                    wave_mgr.current_level += 1; // On passe au niveau +1.
                    wave_mgr.current_wave = 1; // On revient à la vague 1.
                }
                wave_mgr.enemies_spawned = 0; // On remet le compteur d'aliens à zéro.
                wave_mgr.enemies_killed_by_player = 0; // On remet le compteur de tes frags à zéro.
                wave_mgr.state = WaveState::Spawning; // Et on fait revenir des aliens !
            }
        }
    }
}

fn player_control_system(kb: Res<ButtonInput<KeyCode>>, window_q: Query<&Window, With<PrimaryWindow>>, mut query: Query<(&mut Movable, &mut Transform), With<Player>>, state: Res<GameState>) { // Contrôler ton vaisseau.
    if state.game_over || state.victory { return; } // Si le jeu est fini, on ne bouge plus.
    let Ok(window) = window_q.single() else { return }; // On regarde la taille de la fenêtre.
    let limit = window.width() / 2.0 - PLAYER_SIZE.x / 2.0; // On calcule la limite pour ne pas sortir de l'écran.
    if let Ok((mut movable, mut trans)) = query.single_mut() { // Si ton vaisseau existe...
        let mut dir = 0.0; // On commence par ne pas bouger.
        if kb.pressed(KeyCode::ArrowLeft) { dir -= 1.0; } // Flèche Gauche : on va vers la gauche.
        if kb.pressed(KeyCode::ArrowRight) { dir += 1.0; } // Flèche Droite : on va vers la droite.
        movable.velocity.x = dir * PLAYER_SPEED; // On donne la vitesse horizontale.
        trans.translation.x = trans.translation.x.clamp(-limit, limit); // On t'empêche de sortir du cadre.
    }
}

fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>) { // Le moteur qui fait tout bouger.
    for (entity, movable, mut trans) in query.iter_mut() { // Pour chaque objet qui peut bouger...
        trans.translation += movable.velocity.extend(0.0) * time.delta_secs(); // On change sa position selon sa vitesse.
        if trans.translation.y.abs() > 800.0 || trans.translation.x.abs() > 1000.0 { // Si l'objet sort loin de l'écran...
            if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); } // On le supprime pour ne pas ralentir l'ordi.
        }
    }
}

fn enemy_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>, mut enemy_q: Query<(&Transform, &mut EnemyFireTimer)>, player_q: Query<&Transform, With<Player>>) { // Les aliens ripostent !
    let Ok(p_trans) = player_q.single() else { return }; // On regarde où tu es pour te viser.
    for (e_trans, mut timer) in enemy_q.iter_mut() { // Pour chaque alien...
        timer.0.tick(time.delta()); // On fait avancer son chrono de tir.
        if timer.0.just_finished() { // S'il doit tirer...
            let dir = (p_trans.translation - e_trans.translation).xy().normalize_or_zero(); // On vise ta direction.
            commands.spawn(( // On crée la balle alien.
                Bullet { from_player: false }, // Elle vient d'un méchant.
                Movable { velocity: dir * (ENEMY_SPEED * 1.8) }, // Elle fonce vers toi !
                Sprite { image: asset_server.load("sprites/bullet_02.png"), custom_size: Some(BULLET_SIZE), color: Color::srgb(1.0, 0.0, 0.0), ..default() }, // Elle est rouge !
                Transform::from_translation(e_trans.translation), // Elle part de l'alien.
            ));
        }
    }
}
fn player_shoot_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    kb: Res<ButtonInput<KeyCode>>, 
    mut query: Query<(&Transform, &mut PlayerStats), With<Player>>,
    time: Res<Time>,
) {
    let Ok((transform, mut stats)) = query.single_mut() else { return };
    let base_pos = transform.translation + Vec3::new(0.0, 20.0, 0.0);

    // Tir instantané au clic
    if kb.just_pressed(KeyCode::Space) {
        match stats.weapon {
            WeaponMode::Single => spawn_bullet(&mut commands, &asset_server, base_pos, Vec2::new(0.0, BULLET_SPEED)),
            WeaponMode::DoubleJumelé => {
                spawn_bullet(&mut commands, &asset_server, base_pos + Vec3::new(-10.0, 0.0, 0.0), Vec2::new(0.0, BULLET_SPEED));
                spawn_bullet(&mut commands, &asset_server, base_pos + Vec3::new(10.0, 0.0, 0.0), Vec2::new(0.0, BULLET_SPEED));
            },
            WeaponMode::DoubleV => {
                spawn_bullet(&mut commands, &asset_server, base_pos, Vec2::new(-150.0, BULLET_SPEED));
                spawn_bullet(&mut commands, &asset_server, base_pos, Vec2::new(150.0, BULLET_SPEED));
            },
            WeaponMode::Triple | WeaponMode::Quadruple | WeaponMode::Quintuple | WeaponMode::Sixtuple | WeaponMode::Septuple => {
                let count = match stats.weapon {
                    WeaponMode::Triple => 3, WeaponMode::Quadruple => 4,
                    WeaponMode::Quintuple => 5, WeaponMode::Sixtuple => 6,
                    _ => 7,
                };
                for i in 0..count {
                    let step = i as f32 - (count as f32 - 1.0) / 2.0;
                    spawn_bullet(&mut commands, &asset_server, base_pos, Vec2::new(step * 120.0, BULLET_SPEED));
                }
            },
            // Prépare la rafale (bullets une après l'autre)
            WeaponMode::Rapid2 | WeaponMode::Rapid3 | WeaponMode::Rapid4 | WeaponMode::Rapid5 => {
                stats.bullets_left_to_fire = match stats.weapon {
                    WeaponMode::Rapid2 => 2, WeaponMode::Rapid3 => 3, WeaponMode::Rapid4 => 4, _ => 5,
                };
            }
        }
    }

    // Gestion automatique des rafales
    if stats.bullets_left_to_fire > 0 {
        stats.rapid_fire_timer.tick(time.delta());
        if stats.rapid_fire_timer.just_finished() {
            spawn_bullet(&mut commands, &asset_server, base_pos, Vec2::new(0.0, BULLET_SPEED));
            stats.bullets_left_to_fire -= 1;
        }
    }
}

// Fonction utilitaire indispensable
fn spawn_bullet(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec3, vel: Vec2) {
    commands.spawn((
        Bullet { from_player: true },
        Movable { velocity: vel },
        Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() },
        Transform::from_translation(pos),
    ));
}

fn collision_system(
    mut commands: Commands, 
    mut state: ResMut<GameState>, 
    mut wave_mgr: ResMut<WaveManager>,
    bullet_q: Query<(Entity, &Transform, &Bullet)>, 
    enemy_q: Query<(Entity, &Transform, &Enemy)>, 
    mut player_q: Query<(Entity, &Transform, &mut Health, &mut PlayerStats), With<Player>>, 
    powerup_q: Query<(Entity, &Transform, &PowerUp)>, // <--- IMPORTANT
    asset_server: Res<AssetServer>,
) {
    let Ok((p_ent, p_trans, mut p_health, mut p_stats)) = player_q.single_mut() else { return };
    let p_pos = p_trans.translation.xy();

    // 1. RAMASSAGE DES BONUS (Carrés jaunes)
    for (pu_ent, pu_trans, pu_info) in powerup_q.iter() {
        if p_pos.distance(pu_trans.translation.xy()) < 25.0 {
            match pu_info.kind {
                BonusType::Weapon(w) => p_stats.weapon = w, // Changement d'arme
                BonusType::ExtraLife => p_health.current += 1, // +1 Vie
                BonusType::NextLevel => { // Skip Level
                    wave_mgr.state = WaveState::LevelCompleted;
                    wave_mgr.wave_timer.reset();
                }
            }
            commands.entity(pu_ent).despawn(); // Détruit le carré jaune
        }
    }

    // 2. LOGIQUE EXISTANTE (Aliens et Balles)
    for (e_ent, e_trans, e_info) in enemy_q.iter() {
        let e_pos = e_trans.translation.xy();
        let hit_radius = if e_info.kind == EnemyType::Boss { 50.0 } else { 25.0 };

        // Si alien touche joueur
        if p_pos.distance(e_pos) < hit_radius {
            if let Ok(mut cmd) = commands.get_entity(e_ent) { cmd.despawn(); }
            p_health.current -= 1;
            spawn_explosion(&mut commands, &asset_server, p_trans.translation);
            if p_health.current <= 0 { if let Ok(mut cmd) = commands.get_entity(p_ent) { cmd.despawn(); } state.game_over = true; }
        }

        for (b_ent, b_trans, b_type) in bullet_q.iter() {
            let b_pos = b_trans.translation.xy();
            if b_type.from_player && b_pos.distance(e_pos) < hit_radius {
                // MORT D'UN ALIEN -> CHANCE DE BONUS
                if rand::random::<f32>() < 0.2 { // 20% de chance
                    let random_weapon = match rand::random::<u32>() % 8 {
                        0 => WeaponMode::DoubleV, 1 => WeaponMode::Triple, 2 => WeaponMode::Septuple,
                        3 => WeaponMode::Rapid3, 4 => WeaponMode::DoubleJumelé, _ => WeaponMode::Quintuple,
                    };
                    
                    // On choisit au hasard entre arme, vie ou skip level
                    let kind = match rand::random::<u32>() % 10 {
                        0..=7 => BonusType::Weapon(random_weapon),
                        8 => BonusType::ExtraLife,
                        _ => BonusType::NextLevel,
                    };

                    commands.spawn((
                        PowerUp { kind },
                        Movable { velocity: Vec2::new(0.0, -150.0) },
                        Sprite { color: Color::srgb(1.0, 1.0, 0.0), custom_size: Some(Vec2::splat(15.0)), ..default() },
                        Transform::from_translation(e_trans.translation),
                    ));
                }

                state.score += if e_info.kind == EnemyType::Boss { 100 } else { 10 };
                wave_mgr.enemies_killed_by_player += 1;
                spawn_explosion(&mut commands, &asset_server, e_trans.translation);
                if let Ok(mut cmd) = commands.get_entity(e_ent) { cmd.despawn(); }
                if let Ok(mut cmd) = commands.get_entity(b_ent) { cmd.despawn(); }
            } else if !b_type.from_player && b_pos.distance(p_pos) < 15.0 {
                p_health.current -= 1;
                spawn_explosion(&mut commands, &asset_server, p_trans.translation);
                if let Ok(mut cmd) = commands.get_entity(b_ent) { cmd.despawn(); }
                if p_health.current <= 0 { state.game_over = true; }
            }
        }
    }
}



fn spawn_explosion(commands: &mut Commands, asset_server: &Res<AssetServer>, pos: Vec3) { // Créer un feu d'artifice !
    commands.spawn((
        Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) }, // Ça dure 0.3 secondes.
        Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::splat(60.0)), ..default() }, // Image de l'explosion.
        Transform::from_translation(pos), // Là où ça a pété.
    ));
}

fn cleanup_system(mut commands: Commands, time: Res<Time>, mut explosion_q: Query<(Entity, &mut Explosion)>, mut score_q: Query<(Entity, &mut FloatingScore, &mut Transform)>) { // On nettoie ce qui est fini.
    for (entity, mut explosion) in explosion_q.iter_mut() { // Pour chaque explosion...
        explosion.timer.tick(time.delta()); // On fait avancer son chrono.
        if explosion.timer.just_finished() { if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); } } // Si fini, on l'enlève.
    }
    for (entity, mut score, mut trans) in score_q.iter_mut() { // Pour chaque score flottant...
        score.timer.tick(time.delta()); // On fait avancer son chrono.
        trans.translation.y += 1.5; // On le fait monter doucement vers le haut.
        if score.timer.just_finished() { if let Ok(mut cmd) = commands.get_entity(entity) { cmd.despawn(); } } // Si fini, on l'enlève.
    }
}

fn ui_update_system(state: Res<GameState>, wave_mgr: Res<WaveManager>, app_state: Res<State<AppState>>, player_q: Query<&Health, With<Player>>, mut text_queries: ParamSet<(Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LevelText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) { // Mettre à jour les textes.
    if let Ok(mut text) = text_queries.p0().single_mut() { text.0 = format!("Score: {}", state.score); } // On affiche le nouveau score.
    if let Ok(mut text) = text_queries.p1().single_mut() { text.0 = format!("Lvl: {} Wv: {}", wave_mgr.current_level, wave_mgr.current_wave); } // Le niveau et la vague.
    let hp = player_q.single().map(|h| h.current).unwrap_or(0); // On regarde combien tu as de vies.
    if let Ok(mut text) = text_queries.p2().single_mut() { text.0 = format!("Vies: {}", hp); } // On affiche tes vies.
    if let Ok(mut text) = text_queries.p3().single_mut() { // On met à jour le gros message du milieu.
        if *app_state.get() == AppState::Paused { text.0 = "PAUSE".to_string(); } // Si pause, on écrit "PAUSE".
        else if state.game_over { text.0 = "GAME OVER".to_string(); } // Si perdu, on écrit "GAME OVER".
        else if state.victory { text.0 = "VICTOIRE TOTALE !".to_string(); } // Si gagné, on écrit "VICTOIRE".
        else if wave_mgr.state == WaveState::LevelCompleted { text.0 = format!("LEVEL {} RÉUSSI !", wave_mgr.current_level); } // Si niveau fini.
        else if wave_mgr.state == WaveState::Waiting && wave_mgr.show_good_job { text.0 = "Good Job !!!".to_string(); } // Si tu as bien tué tout le monde.
        else { text.0 = "".to_string(); } // Sinon, on n'écrit rien.
    }
}