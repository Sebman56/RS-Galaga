// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Gemini AI le 2025-12-28 11h28
// ═══════════════════════════════════════════════════════════════════════════/




// ═══════════════════════════════════════════════════════════════════════════
// 🎮 Code source en Rust du jeu Xgalaga selon Claude AI le 2025-12-28
// ═══════════════════════════════════════════════════════════════════════════

// 📦 On importe les outils de Bevy pour faire notre jeu
// C'est comme importer des jouets dans ta chambre pour jouer !
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::math::Vec3Swizzles; // 👈 Indispensable pour les collisions (.xy())

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CONSTANTES DU JEU
// ═══════════════════════════════════════════════════════════════════════════
// Les constantes sont des nombres qui ne changent JAMAIS pendant le jeu
// C'est comme les règles d'un jeu de société : elles restent les mêmes !

// 🏃 Vitesse du joueur (pixels par seconde)
// Plus le nombre est grand, plus ton vaisseau va vite !
const PLAYER_SPEED: f32 = 400.0;

// 💨 Vitesse des balles BLEUES du joueur (pixels par seconde)
// Les balles vont encore plus vite que le vaisseau !
const BULLET_SPEED: f32 = 800.0;

// 👾 Vitesse des ennemis (pixels par seconde)
// Les ennemis sont plus lents que ton vaisseau
const ENEMY_SPEED: f32 = 100.0;

// 🔴 Vitesse des balles ROUGES des ennemis (pixels par seconde)
// Pas aussi rapides que tes balles bleues, mais dangereuses !
const ENEMY_BULLET_SPEED: f32 = 400.0;

// 📏 Taille du vaisseau du joueur (largeur x hauteur en pixels)
// Un pixel = un petit point sur l'écran
const PLAYER_SIZE: Vec2 = Vec2::new(30.0, 15.0);

// 📏 Taille des ennemis (largeur x hauteur en pixels)
const ENEMY_SIZE: Vec2 = Vec2::new(20.0, 20.0);

// 📏 Taille des balles (largeur x hauteur en pixels)
// Les balles sont toutes petites !
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);

// ❤️ Nombre de vies du joueur au début du jeu
// Si tu te fais toucher 3 fois, c'est GAME OVER !
const PLAYER_HEALTH: i32 = 3;

// 💥 Durée d'une explosion (en secondes)
// L'explosion disparaît après 0.3 secondes
const EXPLOSION_DURATION: f32 = 0.3;

// 🌊 CONSTANTES POUR LES VAGUES D'ENNEMIS
// Une "vague" = un groupe d'ennemis qui arrivent ensemble

// 👾 Combien d'ennemis dans chaque vague ?
const ENEMIES_PER_WAVE: usize = 10;

// ⏱️ Temps d'attente entre chaque ennemi (en secondes)
// Ils n'arrivent pas tous en même temps, mais un par un !
const TIME_BETWEEN_SPAWNS: f32 = 0.5;

// ⏰ Temps de pause entre deux vagues (en secondes)
// Après avoir tué tous les ennemis, tu as 5 secondes pour souffler !
const TIME_BETWEEN_WAVES: f32 = 5.0;

// 🔫 Les ennemis tirent toutes les 2 secondes
const ENEMY_SHOOT_INTERVAL: f32 = 2.0;

// ═══════════════════════════════════════════════════════════════════════════
// 📦 COMPOSANTS
// ═══════════════════════════════════════════════════════════════════════════
// Les composants sont comme des étiquettes qu'on colle sur les objets du jeu
// Ça permet de dire "celui-là c'est le joueur", "celui-là c'est un ennemi"

// 🚀 Étiquette pour dire "cet objet, c'est le joueur"
#[derive(Component)]
struct Player;

// 📊 Étiquettes pour les textes du bandeau d'informations
#[derive(Component)]
struct ScoreText;      // Le texte qui affiche le score

#[derive(Component)]
struct LevelText;      // Le texte qui affiche le niveau (vague)

#[derive(Component)]
struct LivesText;      // Le texte qui affiche les vies

// 🔵 Étiquette pour dire "c'est une balle BLEUE du joueur"
#[derive(Component)]
struct PlayerBullet;

// 🔴 Étiquette pour dire "c'est une balle ROUGE d'un ennemi"
#[derive(Component)]
struct EnemyBullet;

// ⏰ Un chronomètre pour savoir quand l'ennemi doit tirer
// Chaque ennemi a son propre chronomètre !
#[derive(Component)]
struct EnemyShootTimer {
    timer: Timer,  // Le chronomètre qui compte les secondes
}

// 👾 Étiquette pour dire "cet objet, c'est un ennemi"
#[derive(Component)]
struct Enemy;

// 🏃 Composant qui donne une vitesse à un objet
// Si un objet a ce composant, il peut bouger !
#[derive(Component)]
struct Movable {
    velocity: Vec2, // La vitesse : dans quelle direction et à quelle vitesse ?
}

// ❤️ Composant qui donne des points de vie à un objet
#[derive(Component)]
struct Health {
    current: i32, // Le nombre de vies restantes (3, 2, 1...)
}

// 💥 Composant pour les explosions
// Les explosions disparaissent après un certain temps
#[derive(Component)]
struct Explosion {
    timer: Timer, // Un chronomètre qui compte combien de temps l'explosion existe
}

// ═══════════════════════════════════════════════════════════════════════════
// 🌊 ÉNUMÉRATIONS POUR LES VAGUES
// ═══════════════════════════════════════════════════════════════════════════
// Une énumération = une liste de choix possibles
// C'est comme choisir entre "vanille", "chocolat" ou "fraise" !

/// 🎯 Direction d'où viennent les ennemis
/// Les ennemis peuvent arriver de 3 côtés différents de l'écran
#[derive(Clone, Copy, Debug, PartialEq)]
enum SpawnDirection {
    Top,    // ⬆️ Les ennemis arrivent du haut et descendent
    Left,   // ⬅️ Les ennemis arrivent de la gauche et vont vers la droite
    Right,  // ➡️ Les ennemis arrivent de la droite et vont vers la gauche
}

/// 🌊 État d'une vague d'ennemis
/// Une vague peut être dans 3 états différents
#[derive(Clone, Copy, Debug, PartialEq)]
enum WaveState {
    Spawning,  // 🐣 Les ennemis sont en train d'apparaître un par un
    Fighting,  // ⚔️ Tous les ennemis sont là, on se bat !
    Waiting,   // ⏰ On attend avant la prochaine vague (pause)
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗃️ RESSOURCES
// ═══════════════════════════════════════════════════════════════════════════
// Les ressources sont comme des tableaux de bord qui gardent des infos importantes

/// 🌊 Gestionnaire de vagues d'ennemis
/// C'est le "chef" qui décide quand et où les ennemis apparaissent
#[derive(Resource)]
struct WaveManager {
    current_wave: u32,              // 🔢 Numéro de la vague actuelle (1, 2, 3...)
    state: WaveState,               // 📊 État actuel (Spawning, Fighting ou Waiting)
    direction: SpawnDirection,      // 🎯 D'où viennent les ennemis cette fois ?
    enemies_spawned: usize,         // 👾 Combien d'ennemis sont déjà apparus ?
    spawn_timer: Timer,             // ⏱️ Chronomètre entre chaque ennemi
    wave_timer: Timer,              // ⏰ Chronomètre entre les vagues
}

// 🎬 Valeurs de départ du WaveManager
// Quand le jeu commence, voilà comment c'est configuré
impl Default for WaveManager {
    fn default() -> Self {
        Self {
            current_wave: 1,                    // On commence à la vague 1
            state: WaveState::Spawning,         // On commence par faire apparaître les ennemis
            direction: SpawnDirection::Top,     // Les premiers ennemis viennent du haut
            enemies_spawned: 0,                 // Aucun ennemi n'est encore apparu
            // Chronomètre qui sonne toutes les 0.5 secondes (pour faire apparaître un ennemi)
            spawn_timer: Timer::from_seconds(TIME_BETWEEN_SPAWNS, TimerMode::Repeating),
            // Chronomètre qui sonne une seule fois après 5 secondes (pour la pause)
            wave_timer: Timer::from_seconds(TIME_BETWEEN_WAVES, TimerMode::Once),
        }
    }
}

impl WaveManager {
    /// 🔄 Passe à la vague suivante
    /// Cette fonction est appelée quand on a tué tous les ennemis
    fn next_wave(&mut self) {
        // On augmente le numéro de vague (1 devient 2, 2 devient 3...)
        self.current_wave += 1;
        
        // On remet le compteur d'ennemis à zéro
        self.enemies_spawned = 0;
        
        // On repasse en mode "faire apparaître les ennemis"
        self.state = WaveState::Spawning;
        
        // 🎲 On choisit la direction selon le numéro de vague
        // C'est comme un cycle qui se répète : Haut, Gauche, Droite, Haut, Gauche, Droite...
        self.direction = match self.current_wave % 3 {
            1 => SpawnDirection::Top,    // Vagues 1, 4, 7, 10... → Haut
            2 => SpawnDirection::Left,   // Vagues 2, 5, 8, 11... → Gauche
            _ => SpawnDirection::Right,  // Vagues 3, 6, 9, 12... → Droite
        };
    }
}

/// 🎮 Tableau de bord du jeu
/// Garde le score et dit si le jeu est terminé
#[derive(Resource, Default)]
struct GameState {
    score: u32,         // 🏆 Le score du joueur (nombre de points)
    game_over: bool,    // ☠️ Est-ce que le jeu est fini ? (true = oui, false = non)
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 SYSTÈME DE DÉMARRAGE
// ═══════════════════════════════════════════════════════════════════════════
// Cette fonction est appelée UNE SEULE FOIS au début du jeu
// Elle crée la caméra, le vaisseau du joueur et le bandeau d'infos

fn setup_game(
    mut commands: Commands,         // L'outil pour créer des objets dans le jeu
    asset_server: Res<AssetServer>, // L'outil pour charger les images
) {
    // 📷 Créer la caméra (pour voir le jeu)
    // Sans caméra, on ne verrait rien !
    commands.spawn(Camera2d);
    
    // 🚀 Créer le vaisseau du joueur
    commands.spawn((
        Player,                             // Étiquette "joueur"
        Movable { velocity: Vec2::ZERO },   // Peut bouger (au début vitesse = 0)
        Health { current: PLAYER_HEALTH },  // A 3 vies
        Sprite {                            // Son apparence visuelle
            image: asset_server.load("sprites/player_01.png"), // L'image du vaisseau
            custom_size: Some(PLAYER_SIZE), // Sa taille
            ..default()                     // Le reste : valeurs par défaut
        },
        // 📍 Position de départ : au milieu en bas de l'écran
        Transform::from_xyz(0.0, -300.0, 0.0),
    ));

    // ═══════════════════════════════════════════════════════════════════
    // 📊 CRÉER LE BANDEAU D'INFORMATIONS EN HAUT DE L'ÉCRAN
    // ═══════════════════════════════════════════════════════════════════
    
    // 📦 Créer le conteneur principal (la barre noire en haut)
    commands
        .spawn(Node {
            width: Val::Percent(100.0),           // Prend toute la largeur de l'écran
            height: Val::Px(50.0),                // 50 pixels de haut
            justify_content: JustifyContent::SpaceBetween, // Espace les éléments
            align_items: AlignItems::Center,      // Les centre verticalement
            padding: UiRect::all(Val::Px(20.0)),  // Marge intérieure de 20 pixels
            ..default()
        })
        .with_child((
            // Fond noir semi-transparent
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // 🏆 SCORE - À gauche
            parent.spawn((
                ScoreText,                      // Étiquette "texte du score"
                Text::new("Score: 0"),          // Le texte initial
                TextFont {                      // Style de police
                    font_size: 30.0,            // Taille 30
                    ..default()
                },
                TextColor(Color::WHITE),        // Couleur blanche
            ));
            
            // 🌊 NIVEAU (VAGUE) - Au centre
            parent.spawn((
                LevelText,                      // Étiquette "texte du niveau"
                Text::new("Vague: 1"),          // Le texte initial
                TextFont {                      // Style de police
                    font_size: 30.0,            // Taille 30
                    ..default()
                },
                TextColor(Color::WHITE),        // Couleur blanche
            ));
            
            // ❤️ VIES - À droite
            parent.spawn((
                LivesText,                      // Étiquette "texte des vies"
                Text::new("Vies: 3"),           // Le texte initial
                TextFont {                      // Style de police
                    font_size: 30.0,            // Taille 30
                    ..default()
                },
                TextColor(Color::WHITE),        // Couleur blanche
            ));
        });
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 SYSTÈME D'ENTRÉE DU JOUEUR
// ═══════════════════════════════════════════════════════════════════════════

/// 🎮 Gère le mouvement du joueur (gauche/droite)
/// Cette fonction écoute les touches du clavier
fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,              // Pour savoir quelles touches sont pressées
    mut player_query: Query<&mut Movable, With<Player>>, // Pour modifier la vitesse du joueur
    game_state: Res<GameState>,                       // Pour savoir si le jeu est fini
) {
    // ⛔ Si le jeu est terminé, on ne fait rien
    if game_state.game_over { 
        return; 
    }
    
    // 🧮 Variable pour savoir dans quelle direction aller
    // -1 = gauche, 0 = immobile, 1 = droite
    let mut direction = 0.0;
    
    // ⬅️ Si on appuie sur flèche gauche OU touche A
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) { 
        direction -= 1.0;  // On va vers la gauche
    }
    
    // ➡️ Si on appuie sur flèche droite OU touche D
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) { 
        direction += 1.0;  // On va vers la droite
    }
    
    // 🔄 On applique la direction au vaisseau
    for mut movable in player_query.iter_mut() { 
        // On multiplie la direction par la vitesse du joueur
        // Par exemple : -1 × 400 = -400 (on va vite vers la gauche)
        movable.velocity.x = direction * PLAYER_SPEED; 
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🔵 SYSTÈME DE TIR DU JOUEUR (LASER BLEU)
// ═══════════════════════════════════════════════════════════════════════════

/// 🔫 Gère le tir du joueur (barre d'espace)
/// Quand tu appuies sur ESPACE, tu tires un laser BLEU !
fn player_shooting(
    mut commands: Commands,                        // Pour créer les balles
    asset_server: Res<AssetServer>,               // Pour charger l'image de la balle
    keyboard: Res<ButtonInput<KeyCode>>,          // Pour savoir si on appuie sur espace
    player_query: Query<&Transform, With<Player>>, // Pour savoir où est le joueur
    game_state: Res<GameState>,                   // Pour savoir si le jeu est fini
) {
    // ⛔ Si le jeu est terminé, on ne peut plus tirer
    if game_state.game_over { 
        return; 
    }
    
    // 🔫 Si on appuie sur la barre d'espace (juste au moment où on l'appuie)
    if keyboard.just_pressed(KeyCode::Space) {
        // Pour chaque joueur (normalement il n'y en a qu'un)
        for player_transform in player_query.iter() {
            // 📍 Calculer où faire apparaître la balle
            // Elle apparaît juste au-dessus du vaisseau
            let spawn_pos = player_transform.translation + Vec3::new(
                0.0,  // Même position X (pas de décalage gauche/droite)
                PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,  // Au-dessus du vaisseau
                0.0   // Même profondeur Z
            );
            
            // 💥 Créer le laser BLEU
            commands.spawn((
                PlayerBullet,                   // 🔵 Étiquette "balle du joueur"
                Movable {                       // Elle peut bouger
                    velocity: Vec2::new(0.0, BULLET_SPEED)  // Elle monte tout droit très vite
                },
                Sprite {                        // Son apparence
                    image: asset_server.load("sprites/laser_blue.png"), // 🔵 Image du laser bleu
                    custom_size: Some(BULLET_SIZE),  // Sa taille
                    ..default()
                },
                Transform::from_translation(spawn_pos),  // Sa position de départ
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🔴 SYSTÈME DE TIR DES ENNEMIS (LASER ROUGE)
// ═══════════════════════════════════════════════════════════════════════════

/// 🔴 Les ennemis tirent des lasers ROUGES vers le joueur !
/// Attention, c'est dangereux ! Il faut les éviter !
fn enemy_shooting(
    mut commands: Commands,           // Pour créer les lasers rouges
    asset_server: Res<AssetServer>,   // Pour charger l'image du laser rouge
    time: Res<Time>,                  // Pour savoir combien de temps s'est écoulé
    mut enemy_query: Query<(&Transform, &mut EnemyShootTimer), With<Enemy>>, // Les ennemis
    player_query: Query<&Transform, With<Player>>,  // Le joueur
    game_state: Res<GameState>,       // Pour savoir si le jeu est fini
) {
    // ⛔ Si le jeu est terminé, les ennemis arrêtent de tirer
    if game_state.game_over { 
        return; 
    }
    
    // 🎯 Vérifier si le joueur existe encore
    // (si tu as perdu toutes tes vies, le joueur n'existe plus !)
    if let Ok(player_transform) = player_query.single() {
        let player_pos = player_transform.translation;  // Position du joueur
        
        // 👾 Pour chaque ennemi vivant
        for (enemy_transform, mut shoot_timer) in enemy_query.iter_mut() {
            // ⏰ Faire avancer le chronomètre de cet ennemi
            shoot_timer.timer.tick(time.delta());
            
            // 🔫 Si le chronomètre a sonné (2 secondes se sont écoulées)
            if shoot_timer.timer.just_finished() {
                let enemy_pos = enemy_transform.translation;  // Position de l'ennemi
                
                // 🎯 Calculer la direction vers le joueur
                // On calcule un vecteur qui pointe vers le joueur
                // .normalize() transforme ce vecteur en longueur 1 (pour avoir une direction)
                // .xy() garde seulement X et Y (on enlève Z qu'on n'utilise pas)
                let direction = (player_pos - enemy_pos).normalize().xy();
                
                // 📍 Position de départ du laser (sous l'ennemi)
                let spawn_pos = enemy_pos + Vec3::new(
                    0.0,  // Même position X que l'ennemi
                    -ENEMY_SIZE.y / 2.0 - BULLET_SIZE.y / 2.0,  // Sous l'ennemi
                    0.0   // Même profondeur Z
                );
                
                // 🔴 Créer le laser ROUGE qui va vers le joueur
                commands.spawn((
                    EnemyBullet,                    // 🔴 Étiquette "balle ennemie"
                    Movable {                       // Elle peut bouger
                        // La vitesse = direction × vitesse
                        // Ça fait que le laser va dans la direction du joueur
                        velocity: direction * ENEMY_BULLET_SPEED
                    },
                    Sprite {                        // Son apparence
                        image: asset_server.load("sprites/laser_red.png"), // 🔴 Image du laser rouge
                        custom_size: Some(BULLET_SIZE),  // Sa taille
                        ..default()
                    },
                    Transform::from_translation(spawn_pos),  // Sa position de départ
                ));
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🌊 SYSTÈME DE GESTION DES VAGUES D'ENNEMIS
// ═══════════════════════════════════════════════════════════════════════════

/// 🌊 Gère l'apparition des vagues d'ennemis
/// C'est comme un chef d'orchestre qui dit aux ennemis quand entrer sur scène !
fn wave_spawner(
    mut commands: Commands,                           // Pour créer les ennemis
    asset_server: Res<AssetServer>,                  // Pour charger les images
    time: Res<Time>,                                 // Pour savoir combien de temps s'est écoulé
    mut wave_manager: ResMut<WaveManager>,           // Le gestionnaire de vagues
    enemy_query: Query<&Enemy>,                      // Pour compter combien d'ennemis sont vivants
    window_query: Query<&Window, With<PrimaryWindow>>, // Pour connaître la taille de l'écran
    game_state: Res<GameState>,                      // Pour savoir si le jeu est fini
) {
    // ⛔ Si le jeu est terminé, on arrête tout
    if game_state.game_over { 
        return; 
    }
    
    // 📏 Récupérer la fenêtre du jeu (pour connaître sa taille)
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    
    // 🔢 Compter combien d'ennemis sont encore vivants
    let enemy_count = enemy_query.iter().count();
    
    // 🎯 Selon l'état actuel de la vague, faire différentes choses
    match wave_manager.state {
        // 📍 État SPAWNING : on fait apparaître les ennemis un par un
        WaveState::Spawning => {
            // ⏱️ Faire avancer le chronomètre
            wave_manager.spawn_timer.tick(time.delta());
            
            // ✅ Si le chronomètre a sonné ET qu'il reste des ennemis à faire apparaître
            if wave_manager.spawn_timer.just_finished() 
                && wave_manager.enemies_spawned < ENEMIES_PER_WAVE {
                
                // 🐣 Faire apparaître un nouvel ennemi
                spawn_enemy_from_direction(
                    &mut commands,
                    &asset_server,
                    window,
                    wave_manager.direction,              // D'où il vient
                    wave_manager.enemies_spawned,        // Son numéro dans la vague
                );
                
                // 📈 On augmente le compteur d'ennemis apparus
                wave_manager.enemies_spawned += 1;
                
                // 🎉 Si tous les ennemis sont apparus, on passe à l'état suivant
                if wave_manager.enemies_spawned >= ENEMIES_PER_WAVE { 
                    wave_manager.state = WaveState::Fighting; 
                }
            }
        }
        
        // ⚔️ État FIGHTING : on attend que le joueur tue tous les ennemis
        WaveState::Fighting => {
            // ✅ S'il n'y a plus aucun ennemi vivant
            if enemy_count == 0 {
                // 🎊 La vague est terminée ! On passe en mode attente
                wave_manager.state = WaveState::Waiting;
                
                // ⏰ On remet le chronomètre de pause à zéro
                wave_manager.wave_timer.reset();
            }
        }
        
        // ⏰ État WAITING : pause avant la prochaine vague
        WaveState::Waiting => {
            // ⏱️ Faire avancer le chronomètre de pause
            wave_manager.wave_timer.tick(time.delta());
            
            // ✅ Si la pause est terminée (5 secondes se sont écoulées)
            if wave_manager.wave_timer.is_finished() { 
                // 🌊 On lance la vague suivante !
                wave_manager.next_wave(); 
            }
        }
    }
}

/// 🎯 Fait apparaître un ennemi depuis une direction donnée
/// Cette fonction décide où placer l'ennemi et dans quelle direction il va
fn spawn_enemy_from_direction(
    commands: &mut Commands,         // Pour créer l'ennemi
    asset_server: &Res<AssetServer>, // Pour charger son image
    window: &Window,                 // Pour connaître la taille de l'écran
    direction: SpawnDirection,       // D'où il vient (Haut, Gauche ou Droite)
    index: usize,                    // Son numéro dans la vague (0, 1, 2... 9)
) {
    // 📍 Calculer la position de départ et la vitesse selon la direction
    let (position, velocity) = match direction {
        // ⬇️ Haut : les ennemis descendent du haut de l'écran
        SpawnDirection::Top => {
            // Calculer jusqu'où les ennemis peuvent aller à gauche/droite
            let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
            
            // Position X aléatoire (rand::random donne un nombre entre 0 et 1)
            // On le transforme en position entre -max_x et +max_x
            let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x;
            
            // Position Y : juste au-dessus de l'écran
            let y_pos = window.height() / 2.0 + ENEMY_SIZE.y;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),   // Où il apparaît
                Vec2::new(0.0, -ENEMY_SPEED),   // Il descend (vitesse Y négative)
            )
        }
        
        // ➡️ Gauche : les ennemis viennent de la gauche et vont vers la droite
        SpawnDirection::Left => {
            // 🛡️ Calculer où est le joueur pour ne pas aller trop bas
            // On veut que les ennemis restent AU-DESSUS du joueur
            // Le joueur est à y = -300, donc on limite à y = -250 (50 pixels au-dessus)
            let min_safe_y = -250.0;  // Les ennemis ne vont pas plus bas que ça
            
            // Calculer jusqu'où les ennemis peuvent aller en HAUT
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            
            // 📏 Calculer la hauteur utilisable (entre min_safe_y et max_y)
            let usable_height = max_y - min_safe_y;
            
            // Espacer les ennemis verticalement dans cette zone sécurisée
            // Le premier est à min_safe_y, le dernier est tout en haut
            // On divise la zone sécurisée en 10 parts égales pour les 10 ennemis
            let y_pos = min_safe_y + (index as f32 * usable_height / ENEMIES_PER_WAVE as f32);
            
            // Position X : juste à gauche de l'écran (en dehors, invisible)
            let x_pos = -window.width() / 2.0 - ENEMY_SIZE.x;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),  // Où il apparaît
                Vec2::new(ENEMY_SPEED, 0.0),   // Il va vers la droite (vitesse X positive)
            )
        }
        
        // ⬅️ Droite : les ennemis viennent de la droite et vont vers la gauche
        SpawnDirection::Right => {
            // 🛡️ Même protection : les ennemis restent au-dessus du joueur
            let min_safe_y = -250.0;  // Les ennemis ne vont pas plus bas que ça
            
            // Calculer jusqu'où les ennemis peuvent aller en HAUT
            let max_y = window.height() / 2.0 - ENEMY_SIZE.y / 2.0;
            
            // 📏 Calculer la hauteur utilisable
            let usable_height = max_y - min_safe_y;
            
            // Espacer les ennemis verticalement dans cette zone sécurisée
            let y_pos = min_safe_y + (index as f32 * usable_height / ENEMIES_PER_WAVE as f32);
            
            // Position X : juste à droite de l'écran (en dehors, invisible)
            let x_pos = window.width() / 2.0 + ENEMY_SIZE.x;
            
            (
                Vec3::new(x_pos, y_pos, 0.0),   // Où il apparaît
                Vec2::new(-ENEMY_SPEED, 0.0),   // Il va vers la gauche (vitesse X négative)
            )
        }
    };

    // 🎲 Créer un délai aléatoire pour que tous les ennemis ne tirent pas en même temps
    // rand::random donne un nombre entre 0 et 1
    // On multiplie par l'intervalle de tir (2 secondes)
    let random_delay = rand::random::<f32>() * ENEMY_SHOOT_INTERVAL;
    
    // 👾 Créer l'ennemi avec toutes ses caractéristiques
    commands.spawn((
        Enemy,                              // Étiquette "ennemi"
        Movable { velocity },               // Il peut bouger avec sa vitesse
        EnemyShootTimer {                   // ⏰ Son chronomètre pour tirer
            timer: Timer::from_seconds(
                random_delay + ENEMY_SHOOT_INTERVAL,  // Délai avant le premier tir
                TimerMode::Repeating                   // Se répète à l'infini
            ),
        },
        Sprite {                            // Son apparence
            image: asset_server.load("sprites/enemy_01.png"), // Son image
            custom_size: Some(ENEMY_SIZE),  // Sa taille
            ..default()
        },
        Transform::from_translation(position), // Sa position de départ
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈME DE CRÉATION D'EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════

/// 💥 Crée une explosion à un endroit donné
/// Les explosions sont jolies mais disparaissent vite !
fn spawn_explosion(
    commands: &mut Commands,         // Pour créer l'explosion
    asset_server: &Res<AssetServer>, // Pour charger l'image
    position: Vec3,                  // Où créer l'explosion
    size: Vec2,                      // Quelle taille pour l'explosion
) {
    // 💥 Créer l'explosion
    commands.spawn((
        Explosion {
            // ⏱️ Un chronomètre qui compte 0.3 secondes
            timer: Timer::from_seconds(EXPLOSION_DURATION, TimerMode::Once),
        },
        Sprite {                            // Son apparence
            image: asset_server.load("sprites/explosion_01.png"), // L'image d'explosion
            custom_size: Some(size),        // Sa taille
            ..default()
        },
        Transform::from_translation(position), // Où elle apparaît
    ));
}

// ═══════════════════════════════════════════════════════════════════════════
// 🏃 SYSTÈME DE MOUVEMENT
// ═══════════════════════════════════════════════════════════════════════════

/// 🏃 Fait bouger tous les objets qui ont une vitesse
/// Cette fonction est appelée à chaque image du jeu (60 fois par seconde !)
/// Elle déplace : le joueur, les ennemis, et TOUTES les balles (bleues ET rouges)
fn apply_movement(
    mut query: Query<(&Movable, &mut Transform)>, // Tous les objets qui peuvent bouger
    time: Res<Time>,                               // Pour savoir combien de temps s'est écoulé
) {
    // Pour chaque objet qui peut bouger
    for (movable, mut transform) in query.iter_mut() {
        // 🧮 Calculer de combien il doit bouger
        // On multiplie la vitesse par le temps écoulé depuis la dernière image
        // Par exemple : 400 pixels/sec × 0.016 sec = 6.4 pixels
        let movement = movable.velocity * time.delta_secs();
        
        // ➡️ Déplacer l'objet
        // .extend(0.0) transforme le Vec2 (2D) en Vec3 (3D) en ajoutant Z=0
        transform.translation += movement.extend(0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚧 SYSTÈME DE CONTRAINTES
// ═══════════════════════════════════════════════════════════════════════════

/// 🚧 Empêche le joueur de sortir de l'écran
/// Sans ça, ton vaisseau pourrait partir dans l'espace et disparaître !
fn clamp_player_position(
    mut player_query: Query<&mut Transform, With<Player>>, // Position du joueur
    window_query: Query<&Window, With<PrimaryWindow>>,     // Taille de l'écran
) {
    // 📏 Récupérer la fenêtre
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    
    // 🧮 Calculer jusqu'où le joueur peut aller à gauche/droite
    // On enlève la moitié de la taille du vaisseau pour qu'il ne dépasse pas
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;
    
    // Pour chaque joueur (il n'y en a qu'un normalement)
    for mut transform in player_query.iter_mut() { 
        // 🚧 Limiter sa position entre -limit_x et +limit_x
        // .clamp() force le nombre à rester dans ces limites
        // Par exemple : 500.clamp(-200, 200) = 200 (pas plus que 200 !)
        transform.translation.x = transform.translation.x.clamp(-limit_x, limit_x); 
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗑️ SYSTÈME DE NETTOYAGE
// ═══════════════════════════════════════════════════════════════════════════

/// 🗑️ Supprime les objets qui sont sortis de l'écran
/// Sinon, les balles et ennemis continueraient à voler dans l'espace pour toujours !
/// Cette fonction gère maintenant les balles BLEUES et ROUGES séparément
fn despawn_out_of_bounds(
    mut commands: Commands,  // Pour supprimer des objets
    query: Query<(
        Entity,                // L'objet lui-même
        &Transform,            // Sa position
        Option<&PlayerBullet>, // Est-ce une balle bleue du joueur ?
        Option<&EnemyBullet>,  // Est-ce une balle rouge d'un ennemi ?
        Option<&Enemy>         // Est-ce un ennemi ?
    )>,
    window_query: Query<&Window, With<PrimaryWindow>>, // Taille de l'écran
) {
    // 📏 Récupérer la fenêtre
    let window = window_query.single().expect("Impossible d'obtenir la fenêtre");
    
    // 📏 Calculer les bords de l'écran avec une petite marge
    // On ajoute 50 pixels de marge pour que les objets disparaissent vraiment hors de vue
    let top_edge = window.height() / 2.0 + 50.0;      // Bord du haut
    let bottom_edge = -window.height() / 2.0 - 50.0;  // Bord du bas
    let left_edge = -window.width() / 2.0 - 50.0;     // Bord de gauche
    let right_edge = window.width() / 2.0 + 50.0;     // Bord de droite
    
    // Pour chaque objet dans le jeu
    for (entity, transform, is_player_bullet, is_enemy_bullet, is_enemy) in query.iter() {
        let pos = transform.translation; // Sa position actuelle
        
        // 🔵 Pour les balles BLEUES du joueur : elles montent et sortent EN HAUT
        if is_player_bullet.is_some() && pos.y > top_edge { 
            commands.entity(entity).despawn(); // Supprimer la balle bleue
        }
        
        // 🔴 Pour les balles ROUGES des ennemis : elles descendent et sortent EN BAS
        if is_enemy_bullet.is_some() && pos.y < bottom_edge { 
            commands.entity(entity).despawn(); // Supprimer la balle rouge
        }
        
        // 👾 Pour les ennemis : ils peuvent sortir par N'IMPORTE QUEL bord
        if is_enemy.is_some() {
            // Si l'ennemi est sorti en haut, en bas, à gauche OU à droite
            if pos.y < bottom_edge || pos.y > top_edge 
                || pos.x < left_edge || pos.x > right_edge { 
                commands.entity(entity).despawn(); // Supprimer l'ennemi
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈMES DE COLLISION
// ═══════════════════════════════════════════════════════════════════════════
// Les collisions vérifient si deux objets se touchent
// C'est comme vérifier si deux rectangles se chevauchent !

/// 🔵💥 Collision : Balle BLEUE du joueur touche un ENNEMI
/// Quand ça arrive : l'ennemi explose, la balle disparaît, tu gagnes 10 points !
fn player_bullet_enemy_collision(
    mut commands: Commands,                               // Pour supprimer les objets
    asset_server: Res<AssetServer>,                      // Pour créer des explosions
    bullet_query: Query<(Entity, &Transform), With<PlayerBullet>>, // Toutes les balles bleues
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,         // Tous les ennemis
    mut game_state: ResMut<GameState>,                   // Pour augmenter le score
) {
    // Pour chaque balle bleue
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        // 📍 Position de la balle (on prend juste X et Y, pas Z)
        let bullet_pos = bullet_transform.translation.xy();
        
        // Pour chaque ennemi
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            // 📍 Position de l'ennemi
            let enemy_pos = enemy_transform.translation.xy();
            
            // 🧮 Vérifier s'ils se touchent (collision simple rectangle vs rectangle)
            // On calcule si les distances sont assez petites
            let collision_x = (bullet_pos.x - enemy_pos.x).abs() < (BULLET_SIZE.x + ENEMY_SIZE.x) / 2.0;
            let collision_y = (bullet_pos.y - enemy_pos.y).abs() < (BULLET_SIZE.y + ENEMY_SIZE.y) / 2.0;
            
            // ✅ S'ils se touchent sur X ET sur Y, c'est une collision !
            if collision_x && collision_y {
                // 💥 Créer une belle explosion à l'endroit de l'ennemi
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,    // À la position de l'ennemi
                    ENEMY_SIZE * 1.5,               // Un peu plus grande que l'ennemi
                );
                
                // 🗑️ Supprimer l'ennemi ET la balle
                commands.entity(enemy_entity).despawn();
                commands.entity(bullet_entity).despawn();
                
                // 🎉 Gagner 10 points !
                game_state.score += 10;
                
                // ⛔ Ne pas vérifier les autres ennemis pour cette balle
                // (elle est déjà détruite)
                break;
            }
        }
    }
}

/// 🔴💥 Collision : Balle ROUGE d'un ennemi touche le JOUEUR
/// Quand ça arrive : tu perds une vie ! Si tu n'as plus de vies, c'est Game Over !
fn enemy_bullet_player_collision(
    mut commands: Commands,                                      // Pour supprimer des objets
    asset_server: Res<AssetServer>,                             // Pour créer des explosions
    bullet_query: Query<(Entity, &Transform), With<EnemyBullet>>, // Toutes les balles rouges
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>, // Le joueur
    mut game_state: ResMut<GameState>,                          // Pour le game over
) {
    // ⛔ Si le jeu est déjà terminé, on ne fait rien
    if game_state.game_over { 
        return; 
    }
    
    // Pour chaque joueur (il n'y en a qu'un)
    for (player_entity, player_transform, mut health) in player_query.iter_mut() {
        // 📍 Position du joueur
        let player_pos = player_transform.translation.xy();
        
        // Pour chaque balle rouge
        for (bullet_entity, bullet_transform) in bullet_query.iter() {
            // 📍 Position de la balle rouge
            let bullet_pos = bullet_transform.translation.xy();
            
            // 🧮 Vérifier s'ils se touchent
            let collision_x = (player_pos.x - bullet_pos.x).abs() < (PLAYER_SIZE.x + BULLET_SIZE.x) / 2.0;
            let collision_y = (player_pos.y - bullet_pos.y).abs() < (PLAYER_SIZE.y + BULLET_SIZE.y) / 2.0;
            
            // ✅ S'ils se touchent, c'est dangereux !
            if collision_x && collision_y {
                // 🗑️ Détruire la balle rouge
                commands.entity(bullet_entity).despawn();
                
                // 💥 Petite explosion à l'impact
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    bullet_transform.translation,   // À l'endroit de la balle
                    BULLET_SIZE * 3.0,              // 3 fois plus grande que la balle
                );
                
                // 💔 Perdre une vie !
                health.current -= 1;
                
                // ☠️ Si on n'a plus de vies...
                if health.current <= 0 {
                    // 💥💥 GROSSE explosion sur le joueur
                    spawn_explosion(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,  // À l'endroit du joueur
                        PLAYER_SIZE * 2.0,             // Explosion encore plus grande
                    );
                    
                    // 🗑️ Supprimer le joueur
                    commands.entity(player_entity).despawn();
                    
                    // ☠️ Marquer le jeu comme terminé
                    game_state.game_over = true;
                }
                
                // ⛔ Ne pas vérifier les autres balles
                break;
            }
        }
        
        // ⛔ Si le jeu est terminé, arrêter de vérifier
        if game_state.game_over { 
            break; 
        }
    }
}

/// 💔 Collision : Le JOUEUR touche directement un ENNEMI
/// C'est aussi dangereux qu'une balle rouge !
fn player_enemy_collision(
    mut commands: Commands,                                      // Pour supprimer des objets
    asset_server: Res<AssetServer>,                             // Pour créer des explosions
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>, // Le joueur
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,      // Tous les ennemis
    mut game_state: ResMut<GameState>,                          // Pour le game over
) {
    // ⛔ Si le jeu est déjà terminé, on ne fait rien
    if game_state.game_over { 
        return; 
    }
    
    // Pour chaque joueur (il n'y en a qu'un)
    for (player_entity, player_transform, mut health) in player_query.iter_mut() {
        // 📍 Position du joueur
        let player_pos = player_transform.translation.xy();
        
        // Pour chaque ennemi
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            // 📍 Position de l'ennemi
            let enemy_pos = enemy_transform.translation.xy();
            
            // 🧮 Vérifier s'ils se touchent
            let collision_x = (player_pos.x - enemy_pos.x).abs() < (PLAYER_SIZE.x + ENEMY_SIZE.x) / 2.0;
            let collision_y = (player_pos.y - enemy_pos.y).abs() < (PLAYER_SIZE.y + ENEMY_SIZE.y) / 2.0;
            
            // ✅ S'ils se touchent, c'est un accident !
            if collision_x && collision_y {
                // 💥 Explosion sur l'ennemi
                spawn_explosion(
                    &mut commands,
                    &asset_server,
                    enemy_transform.translation,
                    ENEMY_SIZE * 1.5,
                );
                
                // 🗑️ Supprimer l'ennemi
                commands.entity(enemy_entity).despawn();
                
                // 💔 Perdre une vie !
                health.current -= 1;
                
                // ☠️ Si on n'a plus de vies...
                if health.current <= 0 {
                    // 💥💥 GROSSE explosion sur le joueur
                    spawn_explosion(
                        &mut commands,
                        &asset_server,
                        player_transform.translation,
                        PLAYER_SIZE * 2.0,
                    );
                    
                    // 🗑️ Supprimer le joueur
                    commands.entity(player_entity).despawn();
                    
                    // ☠️ Marquer le jeu comme terminé
                    game_state.game_over = true;
                }
                
                // ⛔ Ne pas vérifier les autres ennemis
                break;
            }
        }
        
        // ⛔ Si le jeu est terminé, arrêter de vérifier
        if game_state.game_over { 
            break; 
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🧹 SYSTÈME DE NETTOYAGE DES EXPLOSIONS
// ═══════════════════════════════════════════════════════════════════════════

/// 🧹 Supprime les explosions après un certain temps
/// Sinon elles resteraient à l'écran pour toujours !
fn cleanup_explosions(
    mut commands: Commands,                          // Pour supprimer les explosions
    mut explosion_query: Query<(Entity, &mut Explosion)>, // Toutes les explosions
    time: Res<Time>,                                 // Pour faire avancer le temps
) {
    // Pour chaque explosion
    for (entity, mut explosion) in explosion_query.iter_mut() {
        // ⏱️ Faire avancer son chronomètre
        explosion.timer.tick(time.delta());
        
        // ✅ Si le temps est écoulé (0.3 secondes sont passées)
        if explosion.timer.is_finished() {
            // 🗑️ Supprimer l'explosion
            commands.entity(entity).despawn();
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🖥️ SYSTÈME D'AFFICHAGE DU BANDEAU
// ═══════════════════════════════════════════════════════════════════════════

/// 🖥️ Met à jour les informations affichées dans le bandeau en haut
/// Cette fonction est appelée à chaque image pour garder les infos à jour !
fn display_info(
    wave_manager: Res<WaveManager>,                 // Pour connaître la vague actuelle
    game_state: Res<GameState>,                     // Pour connaître le score
    player_query: Query<&Health, With<Player>>,     // Pour connaître les vies du joueur
    mut score_text_query: Query<&mut Text, With<ScoreText>>,  // Le texte du score
    mut level_text_query: Query<&mut Text, (With<LevelText>, Without<ScoreText>, Without<LivesText>)>, // Le texte du niveau
    mut lives_text_query: Query<&mut Text, (With<LivesText>, Without<ScoreText>, Without<LevelText>)>, // Le texte des vies
) {
    // 🏆 METTRE À JOUR LE SCORE
    // On cherche le texte du score et on le modifie
    if let Ok(mut text) = score_text_query.single_mut() {
        // ** = déréférence double pour modifier directement le texte
        **text = format!("Score: {}", game_state.score);
    }
    
    // 🌊 METTRE À JOUR LE NIVEAU (VAGUE)
    // On cherche le texte du niveau et on le modifie
    if let Ok(mut text) = level_text_query.single_mut() {
        **text = format!("Vague: {}", wave_manager.current_wave);
    }
    
    // ❤️ METTRE À JOUR LES VIES
    // On cherche le texte des vies et on le modifie
    if let Ok(mut text) = lives_text_query.single_mut() {
        // S'il y a un joueur vivant, on affiche ses vies
        if let Some(health) = player_query.iter().next() { 
            **text = format!("Vies: {}", health.current); 
        } else { 
            // Si le joueur est mort, on affiche 0
            **text = "Vies: 0".to_string(); 
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚀 FONCTION PRINCIPALE
// ═══════════════════════════════════════════════════════════════════════════
// C'est ici que tout commence ! C'est le "chef" du jeu !
// Cette fonction configure tout et lance le jeu

fn main() {
    App::new() // Créer une nouvelle application Bevy
        // 🔌 Ajouter tous les plugins de base de Bevy
        // (fenêtre, son, graphismes, clavier, etc.)
        .add_plugins(DefaultPlugins)
        
        // 🎨 Définir la couleur de fond (bleu très foncé, presque noir)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        
        // 📊 Créer le tableau de bord du jeu (score, game over)
        .init_resource::<GameState>()
        
        // 🌊 Créer le gestionnaire de vagues
        .init_resource::<WaveManager>()
        
        // 🎬 Lancer le setup_game au démarrage (une seule fois)
        .add_systems(Startup, setup_game)
        
        // 🔄 Systèmes qui tournent en boucle (à chaque image, 60 fois par seconde)
        // Ces systèmes s'occupent des contrôles et des tirs
        .add_systems(Update, (
            player_input,       // Écouter les touches gauche/droite
            player_shooting,    // 🔵 Écouter la barre d'espace (tir bleu)
            enemy_shooting,     // 🔴 Les ennemis tirent des lasers rouges
            wave_spawner,       // Faire apparaître les vagues d'ennemis
        ))
        
        // 🏃 Systèmes de mouvement (ils se suivent dans l'ordre)
        // .chain() signifie "fais d'abord celui-ci, puis celui-là"
        .add_systems(Update, (
            apply_movement,         // Bouger tous les objets (joueur, ennemis, balles)
            clamp_player_position,  // Empêcher le joueur de sortir
        ).chain())
        
        // 💥 Systèmes de collisions et nettoyage
        .add_systems(Update, (
            despawn_out_of_bounds,              // Supprimer ce qui sort de l'écran
            player_bullet_enemy_collision,      // 🔵 Vérifier si balle bleue touche ennemi
            enemy_bullet_player_collision,      // 🔴 Vérifier si balle rouge touche joueur
            player_enemy_collision,             // Vérifier si joueur touche ennemi directement
            cleanup_explosions,                 // Supprimer les vieilles explosions
            display_info,                       // Mettre à jour le bandeau d'infos
        ))
        
        // 🎮 Lancer le jeu !
        .run();
}

// ═══════════════════════════════════════════════════════════════════════════
// 📚 RÉCAPITULATIF DU JEU
// ═══════════════════════════════════════════════════════════════════════════
//
// 🎮 COMMENT JOUER :
//    - Flèches ← → ou touches A/D : Bouger le vaisseau
//    - Barre d'espace : Tirer des lasers BLEUS
//    - But : Détruire tous les ennemis de chaque vague !
//
// ⚠️ DANGERS :
//    - Les ennemis tirent des lasers ROUGES qui te suivent !
//    - Si un laser rouge te touche : -1 vie 💔
//    - Si tu touches un ennemi directement : -1 vie 💔
//    - Si tu perds 3 vies : GAME OVER ☠️
//
// 🌊 LES VAGUES :
//    - Chaque vague a 10 ennemis
//    - Vagues 1, 4, 7... → Les ennemis viennent DU HAUT
//    - Vagues 2, 5, 8... → Les ennemis viennent DE GAUCHE
//    - Vagues 3, 6, 9... → Les ennemis viennent DE DROITE
//    - Après chaque vague : 5 secondes de pause !
//
// 🎯 SCORING :
//    - Chaque ennemi détruit = +10 points !
//    - Essaie de faire le meilleur score !
//
// 💡 ASTUCES :
//    - Bouge tout le temps pour éviter les lasers rouges !
//    - Les lasers rouges te suivent, alors change de direction souvent
//    - Tire beaucoup pour détruire les ennemis avant qu'ils ne tirent trop
//
// 🎮 AMUSE-TOI BIEN !
// ═══════════════════════════════════════════════════════════════════════════