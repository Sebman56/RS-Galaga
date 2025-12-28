// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-12-28 à 19h12
// ═══════════════════════════════════════════════════════════════════════════

// Réglage de la collision entre aliens et vaisseau joueur


// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust : Mon super jeu de l'espace (Xgalaga)
// ═══════════════════════════════════════════════════════════════════════════

// On va chercher la caisse à outils "Bevy" pour fabriquer le jeu
use bevy::prelude::*;
// On prépare un bouton pour pouvoir fermer la fenêtre du jeu
use bevy::window::PrimaryWindow;
// C'est le signal pour dire "Le jeu s'arrête maintenant !"
use bevy::app::AppExit;
// Une astuce mathématique pour calculer les positions plus facilement
use bevy::math::Vec3Swizzles;

// 🎮 LES RÉGLAGES (Les chiffres magiques)
const PLAYER_SPEED: f32 = 500.0;     // La vitesse de ton vaisseau
const BULLET_SPEED: f32 = 700.0;     // La vitesse de tes balles laser
const ENEMY_SPEED: f32 = 120.0;      // La vitesse des méchants aliens
const PLAYER_SIZE: Vec2 = Vec2::new(35.0, 20.0); // La taille de ton vaisseau
const ENEMY_SIZE: Vec2 = Vec2::new(25.0, 25.0);  // La taille d'un alien
const BULLET_SIZE: Vec2 = Vec2::new(5.0, 15.0);   // La taille d'un petit laser
const PLAYER_HEALTH: i32 = 3;        // Tu as 3 coeurs au début du jeu

// 📦 LES ÉTIQUETTES (On colle des étiquettes sur les objets pour savoir ce qu'ils sont)
#[derive(Component)] struct Player; // Étiquette : "Ça, c'est le joueur"
#[derive(Component)] struct Enemy;  // Étiquette : "Ça, c'est un méchant"
#[derive(Component)] struct Bullet { from_player: bool } // Étiquette : "C'est une balle" (On note si c'est la tienne ou celle d'un alien)
#[derive(Component)] struct Movable { velocity: Vec2 }   // Étiquette : "Cet objet a le droit de bouger"
#[derive(Component)] struct Health { current: i32 }      // Étiquette : "Cet objet peut recevoir des dégâts"
#[derive(Component)] struct EnemyFireTimer(Timer);       // Étiquette : "C'est la montre de l'alien pour savoir quand tirer"
#[derive(Component)] struct Explosion { timer: Timer }   // Étiquette : "C'est une explosion qui va disparaître bientôt"

// Les étiquettes pour les textes écrits sur l'écran
#[derive(Component)] struct ScoreText;   // Étiquette : "Ici on écrit le score"
#[derive(Component)] struct LevelText;   // Étiquette : "Ici on écrit le niveau"
#[derive(Component)] struct LivesText;   // Étiquette : "Ici on écrit tes vies"
#[derive(Component)] struct MainMessage; // Étiquette : "Ici on écrit VICTOIRE ou PERDU"

// 🌊 LES TYPES DE VAGUES (D'où arrivent les aliens ?)
#[derive(Clone, Copy, Debug, PartialEq)] enum SpawnDirection { Top, Left, Right } // Haut, Gauche ou Droite
#[derive(Clone, Copy, Debug, PartialEq)] enum WaveState { Spawning, Fighting, Waiting } // Apparition, Combat ou Attente

// 🗃️ LE CERVEAU DU JEU (Il se rappelle de tout)
#[derive(Resource)]
struct WaveManager {
    current_level: u32,      // Le numéro du niveau actuel
    current_wave: u32,       // La vague d'aliens actuelle (1, 2, 3...)
    state: WaveState,        // Ce que font les aliens en ce moment
    direction: SpawnDirection, // Par où ils vont arriver
    enemies_spawned: usize,  // Combien d'aliens on a déjà fabriqué
    spawn_timer: Timer,      // Le temps entre l'arrivée de deux aliens
    wave_timer: Timer,       // Le temps de repos entre deux vagues
}

// Les réglages au tout début quand tu lances le jeu
impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_level: 1,
            current_wave: 1,
            state: WaveState::Spawning,
            direction: SpawnDirection::Top,
            enemies_spawned: 0,
            spawn_timer: Timer::from_seconds(0.5, TimerMode::Repeating), // Un alien toutes les demi-secondes
            wave_timer: Timer::from_seconds(4.0, TimerMode::Once),        // 4 secondes de pause
        }
    }
}

// Le score et l'état de la partie
#[derive(Resource, Default)]
struct GameState {
    score: u32,          // Ton nombre de points
    game_over: bool,     // Est-ce que tu as perdu ?
    victory: bool,       // Est-ce que tu as gagné ?
    exit_timer: Option<Timer>, // Une montre pour fermer le jeu doucement
}

// 🎬 LES SYSTÈMES (C'est là qu'on donne les ordres !)

// 1. La mise en place (On prépare le terrain)
fn setup_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d); // On pose une caméra pour voir le jeu

    // On fabrique ton vaisseau spatial
    commands.spawn((
        Player, // On lui met l'étiquette Joueur
        Movable { velocity: Vec2::ZERO }, // Il peut bouger (vitesse 0 au début)
        Health { current: PLAYER_HEALTH }, // On lui donne ses 3 coeurs
        Sprite {
            image: asset_server.load("sprites/player_01.png"), // On charge l'image du vaisseau
            custom_size: Some(PLAYER_SIZE), // On lui donne la bonne taille
            ..default()
        },
        Transform::from_xyz(0.0, -300.0, 0.0), // On le place en bas de l'écran
    ));

    // On prépare le bandeau en haut pour afficher tes points
    commands.spawn(Node {
        width: Val::Percent(100.0), // Ça prend toute la largeur
        height: Val::Px(50.0),      // 50 pixels de haut
        justify_content: JustifyContent::SpaceBetween, // On écarte les textes
        align_items: AlignItems::Center, // On centre le texte dans la barre
        padding: UiRect::horizontal(Val::Px(20.0)), // On laisse un peu de place sur les côtés
        ..default()
    }).with_children(|parent| {
        // On écrit le Niveau, le Score et les Vies
        parent.spawn((LevelText, Text::new(""), TextFont { font_size: 20.0, ..default() }));
        parent.spawn((ScoreText, Text::new("Score: 0"), TextFont { font_size: 25.0, ..default() }));
        parent.spawn((LivesText, Text::new("Vies: 3"), TextFont { font_size: 20.0, ..default() }));
    });

    // On prépare le gros message invisible au milieu (pour la fin)
    commands.spawn((
        MainMessage,
        Text::new(""),
        TextFont { font_size: 40.0, ..default() },
        Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() },
    ));
}

// 2. Le contrôle du vaisseau (C'est toi qui pilotes !)
fn player_control_system(kb: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Movable, With<Player>>) {
    if let Ok(mut movable) = query.single_mut() {
        let mut dir = 0.0; // Au début, on ne bouge pas
        if kb.pressed(KeyCode::ArrowLeft) || kb.pressed(KeyCode::KeyA) { dir -= 1.0; } // Gauche !
        if kb.pressed(KeyCode::ArrowRight) || kb.pressed(KeyCode::KeyD) { dir += 1.0; } // Droite !
        movable.velocity.x = dir * PLAYER_SPEED; // On applique la vitesse choisie
    }
}

// 3. Le bouton pour tirer (PAN ! PAN !)
fn player_shoot_system(mut commands: Commands, asset_server: Res<AssetServer>, kb: Res<ButtonInput<KeyCode>>, query: Query<&Transform, With<Player>>) {
    if kb.just_pressed(KeyCode::Space) { // Si tu appuies sur Espace
        if let Ok(transform) = query.single() {
            // On fabrique une balle laser
            commands.spawn((
                Bullet { from_player: true }, // C'est ta balle à toi
                Movable { velocity: Vec2::new(0.0, BULLET_SPEED) }, // Elle monte vers le haut
                Sprite { image: asset_server.load("sprites/bullet_01.png"), custom_size: Some(BULLET_SIZE), ..default() },
                Transform::from_translation(transform.translation + Vec3::new(0.0, 20.0, 0.0)), // Elle part de ton vaisseau
            ));
        }
    }
}

// 4. L'attaque des aliens (Ils sont malins, ils te visent !)
fn enemy_shoot_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    time: Res<Time>, 
    player_q: Query<&Transform, With<Player>>,
    mut enemy_q: Query<(&Transform, &mut EnemyFireTimer), With<Enemy>>
) {
    // L'alien regarde où tu es sur l'écran
    let player_pos = player_q.single().map(|t| t.translation.xy()).ok();

    for (e_trans, mut timer) in enemy_q.iter_mut() {
        timer.0.tick(time.delta()); // La montre de l'alien avance
        if timer.0.just_finished() { // C'est l'heure de tirer !
            let mut shoot_dir = Vec2::new(0.0, -1.0); // Tirer vers le bas par défaut

            if let Some(p_pos) = player_pos {
                // On calcule le chemin pour que la balle aille vers toi
                let to_player = (p_pos - e_trans.translation.xy()).normalize();
                // On rajoute un petit peu de hasard pour que l'alien rate parfois
                let random_offset = Vec2::new((rand::random::<f32>()-0.5)*0.2, (rand::random::<f32>()-0.5)*0.2);
                shoot_dir = (to_player + random_offset).normalize();
            }

            // L'alien lance sa balle laser rouge
            commands.spawn((
                Bullet { from_player: false }, // Ce n'est pas ta balle !
                Movable { velocity: shoot_dir * (ENEMY_SPEED * 1.8) }, // Elle va vers toi
                Sprite {
                    image: asset_server.load("sprites/bullet_02.png"),
                    custom_size: Some(BULLET_SIZE),
                    color: Color::srgb(1.0, 0.2, 0.2), // Elle est rouge pour faire peur
                    ..default()
                },
                Transform::from_translation(e_trans.translation), // Elle part de l'alien
            ));
        }
    }
}

// 5. Faire bouger tout ce petit monde
fn movement_system(mut commands: Commands, mut query: Query<(Entity, &Movable, &mut Transform)>, time: Res<Time>, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.single().expect("Window error"); // On regarde la taille de la fenêtre
    let half_w = window.width() / 2.0; // Milieu largeur
    let half_h = window.height() / 2.0; // Milieu hauteur
    
    for (entity, movable, mut transform) in query.iter_mut() {
        // On fait avancer l'objet selon sa vitesse
        transform.translation += movable.velocity.extend(0.0) * time.delta_secs();
        
        // Empêcher le joueur de sortir des bords à gauche et à droite
        if transform.translation.y < -200.0 && transform.translation.y > -400.0 {
            transform.translation.x = transform.translation.x.clamp(-half_w + 20.0, half_w - 20.0);
        }
        
        // Si un objet sort trop loin de l'écran, on le supprime pour ne pas fatiguer l'ordinateur
        if transform.translation.y.abs() > half_h + 150.0 || transform.translation.x.abs() > half_w + 150.0 {
            commands.entity(entity).despawn();
        }
    }
}

// 6. Organiser les vagues d'aliens (Le chef d'orchestre)
fn wave_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
    time: Res<Time>, 
    mut wave_mgr: ResMut<WaveManager>, 
    mut game_state: ResMut<GameState>, 
    enemy_q: Query<&Enemy>, 
    window_q: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_q.single().expect("Window error");
    let enemy_count = enemy_q.iter().count(); // On compte combien d'aliens il reste

    // On choisit d'où ils arrivent selon le niveau où tu es
    wave_mgr.direction = match (wave_mgr.current_level, wave_mgr.current_wave) {
        (1, 3) => SpawnDirection::Right, 
        (1, 4) => SpawnDirection::Left,
        (2, 1) | (2, 3) | (3, 5) => SpawnDirection::Right,
        (2, 2) | (2, 4) | (3, 2) | (3, 4) => SpawnDirection::Left,
        (3, 3) => SpawnDirection::Top,
        _ => SpawnDirection::Top,
    };

    match wave_mgr.state {
        WaveState::Spawning => { // On est en train de fabriquer des aliens
            wave_mgr.spawn_timer.tick(time.delta());
            if wave_mgr.spawn_timer.just_finished() && wave_mgr.enemies_spawned < 10 {
                
                // On choisit l'image de l'alien selon sa provenance
                let sprite_path = match wave_mgr.direction {
                    SpawnDirection::Left => "sprites/alien_red.png",   // Rouge si vient de gauche
                    SpawnDirection::Right => "sprites/alien_green.png", // Vert si vient de droite
                    SpawnDirection::Top => "sprites/alien_grey.png",    // Gris si vient du haut
                };

                // On calcule sa position de départ et sa direction
                let (start_pos, velocity) = match wave_mgr.direction {
                    SpawnDirection::Top => (Vec3::new((rand::random::<f32>() - 0.5) * window.width() * 0.8, window.height()/2.0 + 20.0, 0.0), Vec2::new(0.0, -ENEMY_SPEED)),
                    SpawnDirection::Left => (Vec3::new(-window.width()/2.0 - 20.0, 200.0, 0.0), Vec2::new(ENEMY_SPEED, -20.0)),
                    SpawnDirection::Right => (Vec3::new(window.width()/2.0 + 20.0, 200.0, 0.0), Vec2::new(-ENEMY_SPEED, -20.0)),
                };

                // On fait apparaître l'alien !
                commands.spawn((
                    Enemy, 
                    Movable { velocity }, 
                    EnemyFireTimer(Timer::from_seconds(2.0, TimerMode::Repeating)), 
                    Sprite { image: asset_server.load(sprite_path), custom_size: Some(ENEMY_SIZE), ..default() },
                    Transform::from_translation(start_pos)
                ));
                wave_mgr.enemies_spawned += 1;
                if wave_mgr.enemies_spawned >= 10 { wave_mgr.state = WaveState::Fighting; } // Quand il y en a 10, on arrête d'en créer
            }
        },
        WaveState::Fighting => if enemy_count == 0 { // Si tu as tué tout le monde
            if wave_mgr.current_wave >= 5 { // Si c'était la 5ème vague
                if wave_mgr.current_level >= 3 { game_state.victory = true; } // Si c'est le niveau 3, tu as gagné le jeu !
                else { // Sinon, on passe au niveau suivant
                    wave_mgr.current_level += 1; wave_mgr.current_wave = 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); 
                }
            } else { // Sinon on passe à la vague suivante
                wave_mgr.current_wave += 1; wave_mgr.state = WaveState::Waiting; wave_mgr.wave_timer.reset(); 
            }
        },
        WaveState::Waiting => { // Petite pause pour souffler
            wave_mgr.wave_timer.tick(time.delta());
            if wave_mgr.wave_timer.is_finished() && !game_state.victory {
                wave_mgr.enemies_spawned = 0; wave_mgr.state = WaveState::Spawning; // On recommence à créer des aliens
            }
        }
    }
}

// 7. Les accidents (BOUM ! Quand deux objets se touchent)
fn collision_system(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    mut state: ResMut<GameState>, 
    bullet_q: Query<(Entity, &Transform, &Bullet)>, 
    enemy_q: Query<(Entity, &Transform), With<Enemy>>, // On récupère la liste des aliens
    mut player_q: Query<(Entity, &Transform, &mut Health), With<Player>> // On récupère ton vaisseau
) {
    // --- 1. On regarde si le joueur touche un alien (Le nouveau bout de code !) ---
    if let Ok((p_ent, p_trans, mut p_health)) = player_q.single_mut() {
        let p_pos = p_trans.translation.xy(); // Position de ton vaisseau
        
        for (e_ent, e_trans) in enemy_q.iter() {
            let e_pos = e_trans.translation.xy(); // Position de l'alien
            
            // Si la distance est plus petite que 30 pixels (CRASH !)
            if p_pos.distance(e_pos) < 30.0 {
                // On fait une explosion à l'endroit de l'impact
                commands.spawn((
                    Explosion { timer: Timer::from_seconds(0.5, TimerMode::Once) },
                    Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(60.0, 60.0)), ..default() },
                    Transform::from_translation(p_trans.translation),
                ));

                commands.entity(e_ent).despawn(); // L'alien est détruit dans le crash
                p_health.current += 1; // Tu perds un coeur !
                
                if p_health.current <= 0 {
                    commands.entity(p_ent).despawn(); // Ton vaisseau disparaît
                    state.game_over = true; // C'est fini !
                }
            }
        }
    }

    // --- 2. On garde la logique des balles (Ton code actuel) ---
    for (b_ent, b_trans, b_type) in bullet_q.iter() {
        let b_pos = b_trans.translation.xy();
        if b_type.from_player {
            for (e_ent, e_trans) in enemy_q.iter() {
                if b_pos.distance(e_trans.translation.xy()) < 25.0 {
                    commands.spawn((
                        Explosion { timer: Timer::from_seconds(0.3, TimerMode::Once) },
                        Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(40.0, 40.0)), ..default() },
                        Transform::from_translation(e_trans.translation),
                    ));
                    commands.entity(e_ent).despawn(); 
                    commands.entity(b_ent).despawn(); 
                    state.score += 10;
                }
            }
        } else if let Ok((p_ent, p_trans, mut p_health)) = player_q.single_mut() { // Utilisation de get_single_mut pour éviter les crashs si le joueur est déjà mort
            if b_pos.distance(p_trans.translation.xy()) < 20.0 {
                commands.spawn((
                    Explosion { timer: Timer::from_seconds(0.5, TimerMode::Once) },
                    Sprite { image: asset_server.load("sprites/explosion_01.png"), custom_size: Some(Vec2::new(50.0, 50.0)), ..default() },
                    Transform::from_translation(p_trans.translation),
                ));
                commands.entity(b_ent).despawn(); 
                p_health.current -= 1;
                if p_health.current <= 0 { commands.entity(p_ent).despawn(); state.game_over = true; }
            }
        }
    }
}
// 8. Nettoyer les explosions (Elles ne durent pas longtemps)
fn cleanup_explosions(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Explosion)>) {
    for (entity, mut explosion) in query.iter_mut() {
        explosion.timer.tick(time.delta()); // L'explosion vieillit
        if explosion.timer.is_finished() { commands.entity(entity).despawn(); } // Pouf ! Elle s'efface
    }
}

// 9. Mettre à jour les textes sur l'écran
fn ui_system(wave_mgr: Res<WaveManager>, game_state: Res<GameState>, player_q: Query<&Health, With<Player>>, mut texts: ParamSet<(Query<&mut Text, With<LevelText>>, Query<&mut Text, With<ScoreText>>, Query<&mut Text, With<LivesText>>, Query<&mut Text, With<MainMessage>>)>) {
    let dir = match wave_mgr.direction { SpawnDirection::Top => "Haut", SpawnDirection::Left => "Gauche", SpawnDirection::Right => "Droite" };
    // On écrit les infos de la vague et du niveau
    for mut t in texts.p0().iter_mut() { **t = format!("Vague: {} ({}) Lvl: {}", wave_mgr.current_wave, dir, wave_mgr.current_level); }
    // On écrit ton score
    for mut t in texts.p1().iter_mut() { **t = format!("Score: {}", game_state.score); }
    // On regarde combien il te reste de coeurs
    let hp = player_q.single().map(|h| h.current).unwrap_or(0);
    for mut t in texts.p2().iter_mut() { **t = format!("Vies: {}", hp); }
    // On affiche les grands messages de fin ou de début de niveau
    for mut t in texts.p3().iter_mut() {
        if game_state.victory { **t = format!("VICTOIRE ! Score: {}\nR: Rejouer | Q: Quitter", game_state.score); }
        else if game_state.game_over { **t = "GAME OVER\nLes aliens continuent...".to_string(); }
        else if wave_mgr.state == WaveState::Waiting && wave_mgr.current_wave == 1 { **t = format!("LEVEL {}", wave_mgr.current_level); }
        else { **t = "".to_string(); }
    }
}

// 10. Gérer le clavier pour Recommencer ou Quitter
fn input_game_system(
    mut commands: Commands, 
    kb: Res<ButtonInput<KeyCode>>, 
    mut game_state: ResMut<GameState>, 
    mut wave_mgr: ResMut<WaveManager>, 
    mut exit: MessageWriter<AppExit>, 
    time: Res<Time>, 
    all_ents: Query<Entity, Or<(With<Player>, With<Enemy>, With<Bullet>)>>, 
    asset_server: Res<AssetServer>
) {
    // Si tu appuies sur R pour rejouer
    if (game_state.victory || game_state.game_over) && kb.just_pressed(KeyCode::KeyR) {
        for e in all_ents.iter() { commands.entity(e).despawn(); } // On nettoie l'écran
        *game_state = GameState::default(); *wave_mgr = WaveManager::default(); // On remet tout à zéro
        // On fait réapparaître ton vaisseau
        commands.spawn((Player, Movable { velocity: Vec2::ZERO }, Health { current: PLAYER_HEALTH }, 
            Sprite { image: asset_server.load("sprites/player_01.png"), custom_size: Some(PLAYER_SIZE), ..default() },
            Transform::from_xyz(0.0, -300.0, 0.0)));
    }

    // Si tu appuies sur Q pour quitter
    if (game_state.victory || game_state.game_over) && kb.just_pressed(KeyCode::KeyQ) {
        game_state.exit_timer = Some(Timer::from_seconds(1.0, TimerMode::Once)); // On attend 1 seconde
        commands.spawn((Text::new("Au revoir."), TextFont { font_size: 60.0, ..default() }, TextColor(Color::srgb(1.0, 0.0, 0.0)),
            Node { position_type: PositionType::Absolute, align_self: AlignSelf::Center, justify_self: JustifySelf::Center, ..default() }));
    }

    // Si la montre de sortie a fini de tourner, on ferme la fenêtre
    if let Some(ref mut timer) = game_state.exit_timer {
        timer.tick(time.delta());
        if timer.just_finished() { exit.write(AppExit::Success); }
    }
}

// 🚀 LE LANCEMENT (Le bouton START pour l'ordinateur)
fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // On installe tous les outils de base
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.05))) // Couleur de l'espace (bleu très foncé)
        .init_resource::<GameState>() // On prépare le score
        .init_resource::<WaveManager>() // On prépare les aliens
        .add_systems(Startup, setup_game) // On lance la mise en place au début
        .add_systems(Update, ( // On lance tout ça en boucle, tout le temps :
            player_control_system, // Écouter tes commandes
            player_shoot_system,  // Regarder si tu tires
            enemy_shoot_system,   // Faire tirer les aliens
            movement_system,      // Tout faire bouger
            wave_system,          // Gérer les vagues
            collision_system,     // Regarder s'il y a des BOUMS
            cleanup_explosions,   // Effacer les vieilles explosions
            ui_system,            // Mettre à jour le texte
            input_game_system,    // Écouter si tu veux quitter
        ))
        .run(); // C'EST PARTI !
}