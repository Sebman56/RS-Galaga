// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon DeepSeek AI le 2025-12-27
// ═══════════════════════════════════════════════════════════════════════════
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 LES RÈGLES DU JEU QUI NE CHANGENT JAMAIS (CONSTANTES)
// ═══════════════════════════════════════════════════════════════════════════
// C'est comme les règles d'un jeu de société que tu ne peux pas changer !

const PLAYER_SPEED: f32 = 400.0;          // 🚀 Ton vaisseau va à 400 pixels par seconde
                                          // (Comme une petite voiture sur l'écran !)
                                          
const BULLET_SPEED: f32 = 800.0;          // 💥 Tes balles sont SUPER rapides !
                                          // (Elles vont deux fois plus vite que ton vaisseau)
                                          
const ENEMY_SPEED: f32 = 100.0;           // 👽 Les méchants aliens sont plus lents
                                          // (C'est plus facile de les esquiver !)
                                          
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);   // 📏 Taille de ton vaisseau
                                                   // (30 pixels de large, 15 de haut)
                                                   
const ENEMY_SIZE: Vec2 = Vec2::new(20.0, 20.0);    // 📏 Taille des aliens
                                                   // (20x20 pixels, comme un petit carré)
                                                   
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);    // 📏 Taille des balles
                                                   // (Très fines mais longues, comme des crayons !)
                                                   
const PLAYER_HEALTH: i32 = 3;              // ❤️ Tu commences avec 3 vies
                                           // (Comme dans Mario quand tu as un champignon !)
                                           
const EXPLOSION_DURATION: f32 = 0.3;       // 💥 Les explosions durent 0.3 secondes
                                           // (C'est comme un feu d'artifice très court !)

// 🌊 LES NOUVELLES RÈGLES POUR LES VAGUES D'ALIENS !
// ═════════════════════════════════════════════════

const ENEMIES_PER_WAVE: usize = 10;        // 👥 10 aliens par vague
                                           // (Comme une équipe de foot avec 10 joueurs !)
                                           
const TIME_BETWEEN_SPAWNS: f32 = 0.5;      // ⏰ Attendre 0.5 seconde entre chaque alien
                                           // (Compte "1... 2..." entre chaque alien !)
                                           
const TIME_BETWEEN_WAVES: f32 = 5.0;       // 🕐 Attendre 5 secondes entre chaque vague
                                           // (Le temps de reprendre ton souffle !)

// ═══════════════════════════════════════════════════════════════════════════
// 🏷️ LES ÉTIQUETTES QU'ON COLLE SUR TOUT (COMPOSANTS)
// ═══════════════════════════════════════════════════════════════════════════
// C'est comme coller des étiquettes sur tes jouets :
// "Voiture rouge", "Lego", "Peluche"...

#[derive(Component)]                       // 🏷️ "Cette étiquette s'appelle Player"
struct Player;                             // 👤 Ça c'est TOI, le héros du jeu !

#[derive(Component)]
struct Bullet;                             // 💥 C'est une balle que tu tires

#[derive(Component)]
struct Enemy;                              // 👽 C'est un méchant alien

#[derive(Component)]
struct Movable {                           // 🏃 Ça peut BOUGER !
    velocity: Vec2,                        //    Velocity = direction + vitesse
                                          //    (Comme une flèche qui montre où aller)
}

#[derive(Component)]
struct Health {                            // ❤️ Ça a des points de vie
    current: i32,                         //    Vie actuelle (compteur de vies)
}

#[derive(Component)]
struct Explosion {                         // 💥 BOUM ! Une explosion
    timer: Timer,                         //    Un minuteur pour la faire disparaître
}

// ═══════════════════════════════════════════════════════════════════════════
// 🆕 LES CHOIX POUR LES VAGUES (ÉNUMÉRATIONS)
// ═══════════════════════════════════════════════════════════════════════════
// C'est comme une liste de possibilités :
// "Qu'est-ce qu'on mange ? 1-Pizza, 2-Pâtes, 3-Hamburger"

/// 🎯 D'où viennent les aliens
#[derive(Clone, Copy, Debug, PartialEq)]
enum SpawnDirection {                      // Liste des possibilités :
    Top,        // ⬇️  Depuis le HAUT de l'écran (ils tombent)
    Left,       // ➡️  Depuis la GAUCHE (ils viennent de la gauche)
    Right,      // ⬅️  Depuis la DROITE (ils viennent de la droite)
}

/// 🌊 Comment se passe une vague
#[derive(Clone, Copy, Debug, PartialEq)]
enum WaveState {                           // Liste des étapes d'une vague :
    Spawning,   // 📍 On FAIT APPARAÎTRE les aliens (un par un)
    Fighting,   // ⚔️ On COMBAT les aliens (ils sont tous là)
    Waiting,    // ⏰ On ATTEND avant la prochaine vague (repos !)
}

// ═══════════════════════════════════════════════════════════════════════════
// 📦 LES CAHIERS OÙ ON ÉCRIT LES SCORES (RESSOURCES)
// ═══════════════════════════════════════════════════════════════════════════
// C'est comme les cahiers du maître à l'école :
// Tout le monde peut les lire, mais seul le maître peut écrire dedans !

/// 🌊 LE CHEF DES VAGUES (WaveManager)
/// Il décide QUAND et COMMENT les aliens arrivent !
#[derive(Resource)]
struct WaveManager {
    current_wave: u32,              // 📊 Numéro de la vague actuelle (1, 2, 3...)
    state: WaveState,               // 🎭 État actuel (on apparaît ? on combat ? on attend ?)
    direction: SpawnDirection,      // 🧭 D'où viennent les aliens (haut/gauche/droite)
    enemies_spawned: usize,         // 🔢 Combien d'aliens sont déjà apparus (0 à 10)
    spawn_timer: Timer,             // ⏰ Minuteur entre chaque alien (toutes les 0.5s)
    wave_timer: Timer,              // 🕐 Minuteur entre les vagues (5 secondes)
}

impl Default for WaveManager {      // Quand on commence le jeu...
    fn default() -> Self {
        Self {
            current_wave: 1,        // On commence à la vague 1
            state: WaveState::Spawning,  // Tout de suite, on fait apparaître !
            direction: SpawnDirection::Top,  // Les premiers viennent du haut
            enemies_spawned: 0,     // Aucun alien encore apparu
            spawn_timer: Timer::from_seconds(TIME_BETWEEN_SPAWNS, TimerMode::Repeating),
                                     // ⏰ Minuteur qui se répète toutes les 0.5s
            wave_timer: Timer::from_seconds(TIME_BETWEEN_WAVES, TimerMode::Once),
                                     // 🕐 Minuteur qui ne tourne qu'une fois (5s)
        }
    }
}

impl WaveManager {
    /// 🔄 PASSER À LA VAGUE SUIVANTE
    /// Comme passer au niveau suivant dans un jeu !
    fn next_wave(&mut self) {
        self.current_wave += 1;      // On augmente le numéro de vague (1→2→3...)
        self.enemies_spawned = 0;    // On remet le compteur d'aliens à zéro
        self.state = WaveState::Spawning;  // On recommence à faire apparaître
        
        // 🎲 On choisit la direction selon le numéro de vague :
        // C'est comme un pattern qui se répète :
        // Vague 1, 4, 7, 10... = Haut   (reste de la division par 3 = 1)
        // Vague 2, 5, 8, 11... = Gauche (reste = 2)
        // Vague 3, 6, 9, 12... = Droite (reste = 0)
        self.direction = match self.current_wave % 3 {
            1 => SpawnDirection::Top,    // Si reste = 1 → Haut
            2 => SpawnDirection::Left,   // Si reste = 2 → Gauche
            _ => SpawnDirection::Right,  // Sinon → Droite
        };
        
        // 📢 On annonce la nouvelle vague dans la console !
        println!("🌊 VAGUE {} - Direction: {:?}", self.current_wave, self.direction);
    }
}

/// 🎯 L'ÉTAT DU JEU (GameState)
/// Comme le tableau des scores au tableau noir !
#[derive(Resource, Default)]
struct GameState {
    score: u32,          // 🏆 Ton score (points gagnés)
    game_over: bool,     // ❌ Le jeu est-il terminé ? (true = oui, false = non)
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 LE DÉMARRAGE DU JEU (COMME APPUYER SUR "START")
// ═══════════════════════════════════════════════════════════════════════════

fn setup_game(
    mut commands: Commands,          // 🛠️ La boîte à outils pour créer des choses
    asset_server: Res<AssetServer>,  // 🎨 Le cartable avec toutes les images
) {
    // 📷 CRÉER LA CAMÉRA
    // Sans caméra, on ne verrait rien ! C'est comme tes yeux dans le jeu.
    commands.spawn(Camera2d);
    
    // 🚀 CRÉER TON VAISSEAU (LE JOUEUR)
    commands.spawn((
        Player,                      // 🏷️ Étiquette "C'est le joueur"
        Movable { velocity: Vec2::ZERO },  // 🏃 Il peut bouger (vitesse 0 au départ)
        Health { current: PLAYER_HEALTH }, // ❤️ Il a 3 vies
        Sprite {                    // 🎨 Son apparence (son dessin)
            image: asset_server.load("sprites/player_01.png"), // 📁 L'image dans assets/
            custom_size: Some(PLAYER_SIZE),  // 📏 On le redimensionne
            ..default()                      // Le reste par défaut
        },
        Transform::from_xyz(0.0, -300.0, 0.0),  // 📍 Position : centre en bas
                                                // x=0 (centre), y=-300 (bas), z=0
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CE QUE TU FAIS AVEC LE CLAVIER (ENTRÉES)
// ═══════════════════════════════════════════════════════════════════════════

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,  // ⌨️ Le clavier (quelles touches sont pressées)
    mut player_query: Query<&mut Movable, With<Player>>, // 🔍 Cherche le joueur
    game_state: Res<GameState>,           // 📊 L'état du jeu (pour vérifier game over)
) {
    // Si le jeu est terminé, on ne fait rien
    // (Comme quand la récréation est finie !)
    if game_state.game_over {
        return;
    }
    
    let mut direction = 0.0;  // 🧭 Direction : -1 = gauche, 0 = immobile, 1 = droite
    
    // ⬅️ FLÈCHE GAUCHE ou A
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;  // On va vers la gauche
    }
    
    // ➡️ FLÈCHE DROITE ou D
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;  // On va vers la droite
    }
    
    // 🔄 APPLIQUER AU JOUEUR
    // (même s'il n'y a qu'un joueur, on fait une boucle)
    for mut movable in player_query.iter_mut() {
        // La formule magique : vitesse = direction × vitesse_max
        movable.velocity.x = direction * PLAYER_SPEED;
        // Exemple : direction = 1 (droite) × 400 = vitesse de 400 vers la droite
    }
}

fn player_shooting(
    mut commands: Commands,           // 🛠️ Boîte à outils pour créer des balles
    asset_server: Res<AssetServer>,   // 🎨 Cartable d'images
    keyboard: Res<ButtonInput<KeyCode>>, // ⌨️ Clavier
    player_query: Query<&Transform, With<Player>>, // 📍 Position du joueur
    game_state: Res<GameState>,       // 📊 État du jeu
) {
    // Si le jeu est fini, on ne peut plus tirer
    if game_state.game_over {
        return;
    }
    
    // ESPACE vient d'être appuyé ?
    // just_pressed = vrai seulement AU MOMENT où on appuie
    // (pas "pressed" sinon on tirerait 60 balles par seconde !)
    if keyboard.just_pressed(KeyCode::Space) {
        // Pour chaque joueur (il n'y en a qu'un)
        for player_transform in player_query.iter() {
            // 📍 Où faire apparaître la balle ?
            // Au-dessus du vaisseau : position du joueur + moitié de sa hauteur
            let spawn_pos = player_transform.translation + Vec3::new(
                0.0,  // Même position X (horizontal)
                PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,  // Juste au-dessus !
                0.0   // Même profondeur
            );
            
            // 💥 CRÉER UNE NOUVELLE BALLE
            commands.spawn((
                Bullet,                     // 🏷️ Étiquette "C'est une balle"
                Movable { 
                    velocity: Vec2::new(0.0, BULLET_SPEED)  // 🚀 Monte tout droit !
                },
                Sprite {
                    image: asset_server.load("sprites/bullet_01.png"), // 🎨 Image
                    custom_size: Some(BULLET_SIZE),  // 📏 Taille
                    ..default()
                },
                Transform::from_translation(spawn_pos),  // 📍 Position calculée
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🆕 LE SYSTÈME DES VAGUES D'ALIENS (LE PLUS INTÉRESSANT !)
// ═══════════════════════════════════════════════════════════════════════════

/// 🌊 LE CHEF D'ORCHESTRE DES VAGUES
/// Il décide QUOI faire à chaque instant !
fn wave_spawner(
    mut commands: Commands,           // 🛠️ Pour créer des aliens
    asset_server: Res<AssetServer>,   // 🎨 Pour les images d'aliens
    time: Res<Time>,                  // ⏱️ Le temps qui passe (comme une horloge)
    mut wave_manager: ResMut<WaveManager>, // 👨‍💼 Le chef des vagues (on peut le modifier)
    enemy_query: Query<&Enemy>,       // 🔍 Combien d'aliens sont encore en vie ?
    window_query: Query<&Window, With<PrimaryWindow>>, // 📺 La fenêtre du jeu
    game_state: Res<GameState>,       // 📊 État du jeu
) {
    // Si le jeu est terminé, on arrête tout
    if game_state.game_over {
        return;
    }
    
    // 📺 Récupérer la fenêtre (pour connaître sa taille)
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    // 🔢 Compter combien d'aliens sont encore en vie
    let enemy_count = enemy_query.iter().count();
    
    // 🎭 SELON L'ÉTAT ACTUEL, ON FAIT DES CHOSES DIFFÉRENTES :
    match wave_manager.state {
        // 📍 ÉTAT 1 : ON FAIT APPARAÎTRE LES ALIENS
        WaveState::Spawning => {
            // ⏰ Faire avancer le minuteur entre aliens
            wave_manager.spawn_timer.tick(time.delta());
            
            // Si le minuteur est fini ET qu'on a pas encore fait 10 aliens...
            if wave_manager.spawn_timer.just_finished() 
                && wave_manager.enemies_spawned < ENEMIES_PER_WAVE {
                
                // 👽 FAIRE APPARAÎTRE UN ALIEN !
                spawn_enemy_from_direction(
                    &mut commands,        // 🛠️ Boîte à outils
                    &asset_server,        // 🎨 Images
                    window,               // 📺 Fenêtre (pour les positions)
                    wave_manager.direction, // 🧭 Direction (haut/gauche/droite)
                    wave_manager.enemies_spawned, // 🔢 Numéro de l'alien
                );
                
                // 📈 Augmenter le compteur d'aliens apparus
                wave_manager.enemies_spawned += 1;
                
                // Si on a fait apparaître les 10 aliens...
                if wave_manager.enemies_spawned >= ENEMIES_PER_WAVE {
                    // On passe à l'état COMBAT !
                    wave_manager.state = WaveState::Fighting;
                    println!("⚔️ Tous les ennemis sont là ! Combattez !");
                }
            }
        }
        
        // ⚔️ ÉTAT 2 : ON COMBAT LES ALIENS
        WaveState::Fighting => {
            // Si plus aucun alien n'est en vie...
            if enemy_count == 0 {
                // On passe à l'attente !
                wave_manager.state = WaveState::Waiting;
                wave_manager.wave_timer.reset();  // 🔄 Redémarrer le minuteur
                println!("✅ Vague {} terminée ! Prochaine vague dans {}s...", 
                    wave_manager.current_wave, TIME_BETWEEN_WAVES);
            }
        }
        
        // ⏰ ÉTAT 3 : ON ATTEND AVANT LA PROCHAINE VAGUE
        WaveState::Waiting => {
            // Faire avancer le minuteur d'attente
            wave_manager.wave_timer.tick(time.delta());
            
            // CORRECTION : Utiliser is_finished() au lieu de finished()
            if wave_manager.wave_timer.is_finished() {
                // 🌊 ON PASSE À LA VAGUE SUIVANTE !
                wave_manager.next_wave();
            }
        }
    }
}

/// 🎯 FAIRE APPARAÎTRE UN ALIEN DEPUIS UNE DIRECTION
/// C'est comme dire : "Un alien apparaît... de la gauche !"
fn spawn_enemy_from_direction(
    commands: &mut Commands,          // 🛠️ Boîte à outils
    asset_server: &Res<AssetServer>,  // 🎨 Images
    window: &Window,                  // 📺 Fenêtre (taille)
    direction: SpawnDirection,        // 🧭 Direction d'où il vient
    index: usize,                     // 🔢 Numéro de l'alien (0 à 9)
) {
    // Selon la direction, on calcule où il apparaît et où il va :
    let (position, velocity) = match direction {
        // ⬇️ DEPUIS LE HAUT : Les aliens tombent
        SpawnDirection::Top => {
            // Calculer jusqu'où ils peuvent apparaître à gauche/droite
            let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
            // Position X aléatoire entre -max_x et +max_x
            let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x;
            // Position Y : tout en haut de l'écran
            let y_pos = window.height() / 2.0 + ENEMY_SIZE.y;
            
            // 📍 Position = (x aléatoire, tout en haut)
            // 🏃 Velocity = (0, -100) → descend tout droit
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(0.0, -ENEMY_SPEED),
            )
        }
        
        // ➡️ DEPUIS LA GAUCHE : Les aliens viennent de la gauche
        SpawnDirection::Left => {
            // Calculer jusqu'où ils peuvent apparaître en haut/bas
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            // Espacer les aliens verticalement (pour qu'ils soient alignés)
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            // Position X : tout à gauche (en dehors de l'écran)
            let x_pos = -window.width() / 2.0 - ENEMY_SIZE.x;
            
            // 📍 Position = (tout à gauche, position Y calculée)
            // 🏃 Velocity = (+100, 0) → va vers la droite
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(ENEMY_SPEED, 0.0),
            )
        }
        
        // ⬅️ DEPUIS LA DROITE : Les aliens viennent de la droite
        SpawnDirection::Right => {
            // Même calcul pour Y
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            let y_pos = -max_y + (index as f32 * (max_y * 2.0) / ENEMIES_PER_WAVE as f32);
            // Position X : tout à droite (en dehors de l'écran)
            let x_pos = window.width() / 2.0 + ENEMY_SIZE.x;
            
            // 📍 Position = (tout à droite, position Y calculée)
            // 🏃 Velocity = (-100, 0) → va vers la gauche
            (
                Vec3::new(x_pos, y_pos, 0.0),
                Vec2::new(-ENEMY_SPEED, 0.0),
            )
        }
    };
    
    // 👽 CRÉER L'ALIEN !
    commands.spawn((
        Enemy,                      // 🏷️ Étiquette "C'est un alien"
        Movable { velocity },       // 🏃 Avec sa vitesse (calculée plus haut)
        Sprite {
            image: asset_server.load("sprites/enemy_01.png"), // 🎨 Image d'alien
            custom_size: Some(ENEMY_SIZE),  // 📏 Taille
            ..default()
        },
        Transform::from_translation(position),  // 📍 À la position calculée
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 FAIRE DES EXPLOSIONS (C'EST JOLI !)
// ═══════════════════════════════════════════════════════════════════════════

fn spawn_explosion(
    commands: &mut Commands,        // 🛠️ Boîte à outils
    asset_server: &Res<AssetServer>, // 🎨 Images
    position: Vec3,                 // 📍 Où ça explose ?
    size: Vec2,                     // 📏 Quelle taille ?
) {
    commands.spawn((
        Explosion {                 // 💥 Étiquette "C'est une explosion"
            timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
            // ⏰ Minuteur : dure 0.3 secondes, puis disparaît
        },
        Sprite {
            image: asset_server.load("sprites/explosion_01.png"), // 🎨 Image d'explosion
            custom_size: Some(size),  // 📏 Taille (parfois plus grande que l'alien)
            ..default()
        },
        Transform::from_translation(position),  // 📍 À l'endroit de l'explosion
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 🏃 TOUT CE QUI BOUGE (MOUVEMENT)
// ═══════════════════════════════════════════════════════════════════════════

fn apply_movement(
    mut query: Query<(&Movable, &mut Transform)>, // 🔍 Tout ce qui a un Movable
    time: Res<Time>,                              // ⏱️ Le temps qui passe
) {
    // Pour chaque chose qui peut bouger (joueur, aliens, balles)...
    for (movable, mut transform) in query.iter_mut() {
        // 🧮 LA FORMULE MAGIQUE DU MOUVEMENT :
        // distance = vitesse × temps_écoulé
        let movement = movable.velocity * time.delta_secs();
        
        // Ajouter cette distance à la position actuelle
        transform.translation += movement.extend(0.0);
        // extend(0.0) transforme Vec2 (x,y) en Vec3 (x,y,z=0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚧 TU NE PEUX PAS SORTIR DE L'ÉCRAN !
// ═══════════════════════════════════════════════════════════════════════════

fn clamp_player_position(
    mut player_query: Query<&mut Transform, With<Player>>, // 🔍 Le joueur
    window_query: Query<&Window, With<PrimaryWindow>>,    // 📺 La fenêtre
) {
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    // Calculer la limite gauche/droite
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;
    // Moitié de l'écran - moitié du vaisseau = bord sans dépasser
    
    // Pour le joueur...
    for mut transform in player_query.iter_mut() {
        // clamp() = forcer entre -limit_x et +limit_x
        // Si X < -limit_x → X = -limit_x
        // Si X > +limit_x → X = +limit_x
        transform.translation.x = transform.translation.x.clamp(-limit_x, limit_x);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗑️ NETTOYER CE QUI SORT DE L'ÉCRAN
// ═══════════════════════════════════════════════════════════════════════════

fn despawn_out_of_bounds(
    mut commands: Commands,  // 🛠️ Pour supprimer des choses
    // 🔍 On cherche :
    // - Entity (l'objet lui-même)
    // - Transform (sa position)
    // - Option<&Bullet> (peut-être une balle ?)
    // - Option<&Enemy> (peut-être un alien ?)
    query: Query<(Entity, &Transform, Option<&Bullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,  // 📺 Fenêtre
) {
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    
    // 📏 Les bords de l'écran avec une marge (50 pixels)
    let top_edge = window.height() / 2.0 + 50.0;      // En haut + marge
    let bottom_edge = -window.height() / 2.0 - 50.0;  // En bas - marge
    let left_edge = -window.width() / 2.0 - 50.0;     // À gauche - marge
    let right_edge = window.width() / 2.0 + 50.0;     // À droite + marge

    // Pour chaque chose dans la requête...
    for (entity, transform, is_bullet, is_enemy) in query.iter() {
        let pos = transform.translation;  // Sa position
        
        // 💥 Si c'est une BALLE et qu'elle est trop haute...
        if is_bullet.is_some() && pos.y > top_edge {
            commands.entity(entity).despawn();  // Pouf ! Disparaît
        }
        
        // 👽 Si c'est un ALIEN et qu'il est hors de l'écran...
        if is_enemy.is_some() {
            // En bas ? En haut ? À gauche ? À droite ?
            if pos.y < bottom_edge || pos.y > top_edge 
                || pos.x < left_edge || pos.x > right_edge {
                commands.entity(entity).despawn();  // Pouf ! Disparaît
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 QUAND LES CHOSES SE PERCUTENT (COLLISIONS)
// ═══════════════════════════════════════════════════════════════════════════

fn bullet_enemy_collision(
    mut commands: Commands,           // 🛠️ Pour supprimer/supprimer
    asset_server: Res<AssetServer>,   // 🎨 Pour les explosions
    bullet_query: Query<(Entity, &Transform), With<Bullet>>, // 💥 Toutes les balles
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,   // 👽 Tous les aliens
    mut game_state: ResMut<GameState>, // 📊 Pour modifier le score
) {
    // Pour chaque BALLE...
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        let bullet_pos = bullet_transform.translation.xy();  // Position 2D de la balle
        let bullet_half = BULLET_SIZE / 2.0;                 // Demi-taille de la balle
        
        // Pour chaque ALIEN...
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();  // Position 2D de l'alien
            let enemy_half = ENEMY_SIZE / 2.0;                 // Demi-taille de l'alien
            
            // 📐 DÉTECTION DE COLLISION (méthode AABB)
            // On vérifie si les rectangles se touchent :
            // 1. Distance horizontale entre centres
            let dx = (bullet_pos.x - enemy_pos.x).abs();
            // 2. Distance verticale entre centres
            let dy = (bullet_pos.y - enemy_pos.y).abs();
            
            // 3. Si les distances sont < somme des demi-tailles → COLLISION !
            let collision = dx < (bullet_half.x + enemy_half.x) 
                         && dy < (bullet_half.y + enemy_half.y);

            // 💥 SI COLLISION...
            if collision {
                // Faire une explosion à la position de l'alien
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,  // Explosion 1.5× plus grosse que l'alien
                );
                
                // Supprimer l'alien et la balle
                commands.entity(enemy_entity).despawn();
                commands.entity(bullet_entity).despawn();
                
                // 🏆 AJOUTER 10 POINTS AU SCORE !
                game_state.score += 10;
                println!("💥 Touché ! Score : {}", game_state.score);
                
                break;  // Cette balle ne peut toucher qu'un alien, on arrête
            }
        }
    }
}

fn player_enemy_collision(
    mut commands: Commands,           // 🛠️ Pour supprimer/exploser
    asset_server: Res<AssetServer>,   // 🎨 Pour les explosions
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>, // 👤 Joueur
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,   // 👽 Aliens
    mut game_state: ResMut<GameState>, // 📊 État du jeu
) {
    // Si déjà game over, on ne fait rien
    if game_state.game_over {
        return;
    }

    // Pour le JOUEUR...
    for (player_entity, player_transform, mut health) in player_query.iter_mut() {
        let player_pos = player_transform.translation.xy();
        let player_half = PLAYER_SIZE / 2.0;

        // Pour chaque ALIEN...
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();
            let enemy_half = ENEMY_SIZE / 2.0;

            // Même détection de collision que pour les balles
            let dx = (player_pos.x - enemy_pos.x).abs();
            let dy = (player_pos.y - enemy_pos.y).abs();
            
            let collision = dx < (player_half.x + enemy_half.x)
                         && dy < (player_half.y + enemy_half.y);

            // 💥 SI COLLISION...
            if collision {
                // Explosion à la position de l'alien
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,
                );
                
                // Supprimer l'alien
                commands.entity(enemy_entity).despawn();
                
                // 💔 PERDRE UNE VIE
                health.current -= 1;
                println!("💔 Aïe ! Vies restantes : {}", health.current);
                
                // ☠️ SI PLUS DE VIES...
                if health.current <= 0 {
                    // Grosse explosion du vaisseau
                    spawn_explosion(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,
                        PLAYER_SIZE * 2.0,  // Grosse explosion !
                    );
                    
                    // Supprimer le vaisseau
                    commands.entity(player_entity).despawn();
                    
                    // GAME OVER !
                    game_state.game_over = true;
                    println!("☠️ GAME OVER ! Score final : {}", game_state.score);
                }
                break;  // On ne vérifie pas les autres aliens
            }
        }
        
        // Si game over, on arrête tout
        if game_state.game_over {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧹 NETTOYER LES EXPLOSIONS (ELLES DISPARAISSENT APRÈS 0.3 SECONDES)
// ═══════════════════════════════════════════════════════════════════════════

fn cleanup_explosions(
    mut commands: Commands,                // 🛠️ Pour supprimer
    mut explosion_query: Query<(Entity, &mut Explosion)>, // 💥 Toutes les explosions
    time: Res<Time>,                       // ⏱️ Le temps qui passe
) {
    // Pour chaque explosion...
    for (entity, mut explosion) in explosion_query.iter_mut() {
        // Faire avancer son minuteur
        explosion.timer.tick(time.delta());
        
        // Si le minuteur est terminé...
        if explosion.timer.is_finished() {
            // Supprimer l'explosion
            commands.entity(entity).despawn();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🖥️ AFFICHER LES INFOS (DANS LE TERMINAL POUR L'INSTANT)
// ═══════════════════════════════════════════════════════════════════════════

fn display_info(
    wave_manager: Res<WaveManager>,  // 🌊 Le chef des vagues (pour connaître la vague)
    game_state: Res<GameState>,      // 📊 Le score et si game over
    player_query: Query<&Health, With<Player>>, // ❤️ La santé du joueur
) {
    // Si le joueur existe encore...
    if let Some(health) = player_query.iter().next() {
        // On pourrait afficher ces infos à l'écran plus tard !
        // Pour l'instant, on les garde juste en mémoire
        let _wave_num = wave_manager.current_wave;  // Numéro de vague
        let _score = game_state.score;              // Score
        let _health = health.current;               // Vies restantes
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚀 LA FONCTION PRINCIPALE (LE BOUTON "START" DU JEU)
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    // 🎮 CRÉER L'APPLICATION BEVY (le moteur du jeu)
    App::new()
        // 🔌 AJOUTER TOUS LES OUTILS PAR DÉFAUT
        // (fenêtre, graphismes, sons, clavier, souris...)
        .add_plugins(DefaultPlugins)
        
        // 🎨 CHOISIR LA COULEUR DE FOND
        // (bleu très foncé, presque noir - comme l'espace !)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        
        // 📊 INITIALISER L'ÉTAT DU JEU
        // (score = 0, game_over = false au départ)
        .init_resource::<GameState>()
        
        // 🌊 INITIALISER LE GESTIONNAIRE DE VAGUES (NOUVEAU !)
        .init_resource::<WaveManager>()
        
        // 🎬 SYSTÈME QUI S'EXÉCUTE UNE SEULE FOIS AU DÉBUT
        .add_systems(Startup, setup_game)
        
        // 🔄 SYSTÈMES QUI S'EXÉCUTENT À CHAQUE FRAME (60 FOIS PAR SECONDE !)
        
        // GROUPE 1 : LES CONTRÔLES (peuvent tourner en même temps)
        .add_systems(Update, (
            player_input,     // ⌨️ Lire le clavier pour bouger
            player_shooting,  // 🔫 Tirer avec espace
            wave_spawner,     // 🌊 Gérer les vagues d'aliens (NOUVEAU !)
        ))
        
        // GROUPE 2 : LA PHYSIQUE (doivent être dans l'ordre)
        .add_systems(Update, (
            apply_movement,         // 🏃 Faire bouger tout ce qui a une vitesse
            clamp_player_position,  // 🚧 Empêcher le joueur de sortir
        ).chain())  // .chain() = "exécuter dans cet ordre, un après l'autre"
        
        // GROUPE 3 : LE RESTE (peuvent tourner en même temps)
        .add_systems(Update, (
            despawn_out_of_bounds,    // 🗑️ Supprimer ce qui sort de l'écran
            bullet_enemy_collision,   // 💥 Collisions balle-alien
            player_enemy_collision,   // 💔 Collisions joueur-alien
            cleanup_explosions,       // 🧹 Nettoyer les explosions
            display_info,             // 🖥️ Afficher les infos
        ))
        
        // 🚀 LANCER LE JEU !
        .run();
}

// ═══════════════════════════════════════════════════════════════════════════
// 📚 PETIT RÉSUMÉ POUR LES ENFANTS DE 10 ANS
// ═══════════════════════════════════════════════════════════════════════════
//
// 🎮 CE JEU, C'EST COMME :
// 1. Tu es un vaisseau spatial en bas de l'écran
// 2. Des aliens arrivent par vagues de 10
// 3. Tu dois les détruire avant qu'ils te touchent
//
// 🌊 LES VAGUES, C'EST COMME :
// 1ère vague : Les aliens tombent du haut (facile !)
// 2ème vague : Ils viennent de la gauche (attention !)
// 3ème vague : Ils viennent de la droite (tricky !)
// Et ça recommence !
//
// 🎯 TU GAGNES DES POINTS QUAND :
// - Tu touches un alien avec une balle : +10 points
//
// 💔 TU PERDS DES VIES QUAND :
// - Un alien te touche : -1 vie
// - Plus de vies = GAME OVER
//
// ⚡ LE TRUC COOL :
// - Le jeu peut faire plusieurs choses EN MÊME TEMPS !
// - Comme avoir 4 copains qui t'aident sur ton devoir
// - Un s'occupe du clavier, un des aliens, un des collisions...
//
// 🎨 POUR LES IMAGES :
// Mets-les dans le dossier "assets/sprites/" :
// - player_01.png   → Ton vaisseau
// - bullet_01.png   → Tes balles
// - enemy_01.png    → Les aliens
// - explosion_01.png → Les explosions
//
// AMUSE-TOI BIEN ! 🚀👽💥