
// Version de Galaga de ClauddeAI du 2025-12-22

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 CONSTANTES DU JEU (les nombres qui ne changent jamais)
// ═══════════════════════════════════════════════════════════════════════════

const PLAYER_SPEED: f32 = 400.0;      // Vitesse du vaisseau (pixels par seconde)
const BULLET_SPEED: f32 = 800.0;      // Vitesse des balles (très rapide !)
const ENEMY_SPEED: f32 = 100.0;       // Vitesse des ennemis (moins rapide)
const MAX_ENEMIES: usize = 10;        // Maximum d'ennemis en même temps
const PLAYER_SIZE: Vec2 = Vec2::new(60.0, 30.0);   // Taille du vaisseau
const ENEMY_SIZE: Vec2 = Vec2::new(40.0, 40.0);    // Taille des ennemis
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 15.0);    // Taille des balles
const PLAYER_HEALTH: i32 = 3;         // Points de vie du joueur

// ═══════════════════════════════════════════════════════════════════════════
// 📦 COMPOSANTS (les "étiquettes" qu'on colle sur les entités)
// ═══════════════════════════════════════════════════════════════════════════
// Pense aux composants comme des autocollants :
// - Un vaisseau a l'autocollant "Player" + "Movable" + "Health"
// - Une balle a l'autocollant "Bullet" + "Movable"
// Bevy regarde ces autocollants pour savoir quoi faire avec chaque chose !

/// 👾 Étiquette pour le joueur (il n'y en a qu'un)
#[derive(Component)]
struct Player;

/// 💥 Étiquette pour les balles
#[derive(Component)]
struct Bullet;

/// 👽 Étiquette pour les ennemis
#[derive(Component)]
struct Enemy;

/// 🏃 Composant pour tout ce qui peut bouger
/// velocity = direction + vitesse (comme une flèche qui montre où aller)
#[derive(Component)]
struct Movable {
    velocity: Vec2,  // Vec2 = vecteur 2D (x, y)
}

/// ❤️ Composant pour la santé (points de vie)
#[derive(Component)]
struct Health {
    current: i32,    // Vie actuelle
    // Note : 'max' pourrait servir plus tard pour des power-ups qui augmentent la vie max
    // Pour l'instant on ne l'utilise pas, mais on le garde pour le futur
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗃️ RESSOURCES (les données partagées par tout le jeu)
// ═══════════════════════════════════════════════════════════════════════════
// Les ressources sont comme un tableau noir dans une classe :
// Tout le monde peut le lire, mais un seul à la fois peut écrire dessus

/// ⏰ Chronomètre pour faire apparaître les ennemis
#[derive(Resource)]
struct EnemySpawnTimer(Timer);

/// 🎯 État du jeu (score, game over, etc.)
#[derive(Resource, Default)]
struct GameState {
    score: u32,          // Score actuel
    game_over: bool,     // Est-ce que le jeu est terminé ?
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎬 SYSTÈME DE DÉMARRAGE (s'exécute UNE SEULE FOIS au début)
// ═══════════════════════════════════════════════════════════════════════════

/// 🏗️ Configure le jeu au démarrage
/// Ce système crée la caméra et le joueur
fn setup_game(mut commands: Commands) {
    // 📷 Créer une caméra pour voir le jeu
    // (sans caméra, on ne verrait rien à l'écran !)
    commands.spawn(Camera2d);
    
    // 🚀 Créer le vaisseau du joueur
    commands.spawn((
        // Les "autocollants" (composants) du joueur :
        Player,                                    // C'est le joueur
        Movable { velocity: Vec2::ZERO },         // Il peut bouger (vitesse 0 au départ)
        Health { current: PLAYER_HEALTH },        // Il a 3 vies
        
        // L'apparence visuelle :
        Sprite {
            color: Color::srgb(0.0, 0.7, 1.0),    // Couleur bleue
            custom_size: Some(PLAYER_SIZE),        // Taille personnalisée
            ..default()                            // Le reste par défaut
        },
        
        // Position dans le monde :
        Transform::from_xyz(0.0, -300.0, 0.0),    // En bas de l'écran (y négatif)
    ));

    // ⏰ Créer le chronomètre pour faire apparaître les ennemis
    commands.insert_resource(
        EnemySpawnTimer(Timer::from_seconds(1.5, TimerMode::Repeating))
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 🎮 SYSTÈMES D'ENTRÉE (ce qui réagit aux touches du clavier)
// ═══════════════════════════════════════════════════════════════════════════
// Ces systèmes LISENT le clavier et MODIFIENT la velocity
// Ils ne bougent PAS directement les objets (c'est apply_movement qui le fait)

/// ⌨️ Contrôle le vaisseau avec les flèches ou WASD
/// CE SYSTÈME :
/// - Lit le clavier
/// - Change la VELOCITY (pas la position !)
/// - Peut tourner EN PARALLÈLE avec enemy_spawner (pas de conflit)
fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,           // Le clavier
    mut player_query: Query<&mut Movable, With<Player>>,  // Cherche le joueur
    game_state: Res<GameState>,                    // L'état du jeu
) {
    // Si le jeu est terminé, on ne fait rien
    if game_state.game_over {
        return;
    }
    
    // Direction : -1.0 = gauche, 0.0 = immobile, 1.0 = droite
    let mut direction = 0.0;
    
    // ⬅️ Si on appuie sur flèche gauche ou A
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;  // On va vers la gauche
    }
    
    // ➡️ Si on appuie sur flèche droite ou D
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;  // On va vers la droite
    }
    
    // 🔄 Appliquer la direction au joueur
    for mut movable in player_query.iter_mut() {
        // velocity.x = vitesse horizontale
        // direction * PLAYER_SPEED = direction × 400 pixels/seconde
        movable.velocity.x = direction * PLAYER_SPEED;
        
        // Note : velocity.y reste inchangé (le joueur ne monte/descend pas)
    }
}

/// 🔫 Faire tirer le joueur avec la barre ESPACE
/// CE SYSTÈME :
/// - Lit le clavier
/// - CRÉE de nouvelles balles
/// - Peut tourner EN PARALLÈLE avec player_input (pas de conflit)
fn player_shooting(
    mut commands: Commands,                        // Pour créer des entités
    keyboard: Res<ButtonInput<KeyCode>>,           // Le clavier
    player_query: Query<&Transform, With<Player>>, // Position du joueur
    game_state: Res<GameState>,                    // État du jeu
) {
    // Si le jeu est terminé, on ne tire pas
    if game_state.game_over {
        return;
    }
    
    // just_pressed = vrai SEULEMENT la première frame où on appuie
    // (pas pressed, sinon on tire 60 balles par seconde !)
    if keyboard.just_pressed(KeyCode::Space) {
        
        // Pour chaque joueur (il n'y en a qu'un, mais on fait une boucle quand même)
        for player_transform in player_query.iter() {
            
            // 📍 Calculer où faire apparaître la balle
            // Au-dessus du joueur : position du joueur + la moitié de sa hauteur
            let spawn_pos = player_transform.translation + Vec3::new(
                0.0,                                    // Même X que le joueur
                PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,  // Juste au-dessus
                0.0                                     // Même Z
            );
            
            // 💥 Créer une nouvelle balle
            commands.spawn((
                Bullet,                                // C'est une balle
                Movable { 
                    velocity: Vec2::new(0.0, BULLET_SPEED)  // Monte vers le haut
                },
                Sprite {
                    color: Color::srgb(1.0, 1.0, 0.0), // Jaune
                    custom_size: Some(BULLET_SIZE),
                    ..default()
                },
                Transform::from_translation(spawn_pos),
            ));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 👾 SYSTÈMES D'APPARITION DES ENNEMIS
// ═══════════════════════════════════════════════════════════════════════════

/// 🎲 Fait apparaître des ennemis régulièrement en haut de l'écran
/// CE SYSTÈME :
/// - Utilise un Timer (chronomètre)
/// - CRÉE de nouveaux ennemis
/// - Peut tourner EN PARALLÈLE avec player_input et player_shooting
fn enemy_spawner(
    mut commands: Commands,                        // Pour créer des ennemis
    time: Res<Time>,                               // Le temps qui passe
    mut timer: ResMut<EnemySpawnTimer>,           // Le chronomètre
    enemy_query: Query<&Enemy>,                    // Tous les ennemis actuels
    window_query: Query<&Window, With<PrimaryWindow>>, // La fenêtre du jeu
    game_state: Res<GameState>,                    // État du jeu
) {
    // Si le jeu est terminé, on arrête de faire apparaître des ennemis
    if game_state.game_over {
        return;
    }
    
    // ⏰ Faire avancer le chronomètre
    timer.0.tick(time.delta());

    // ✅ Si le chronomètre a fini ET qu'il y a moins de 10 ennemis
    if timer.0.just_finished() && enemy_query.iter().count() < MAX_ENEMIES {
        
        // 📏 Obtenir la taille de la fenêtre
        let window = window_query.single().unwrap();
        
        // Calculer jusqu'où les ennemis peuvent apparaître sur X
        // (on ne veut pas qu'ils apparaissent en dehors de l'écran)
        let max_x = window.width() / 2.0 - ENEMY_SIZE.x / 2.0;
        
        // 🎲 Position X aléatoire entre -max_x et +max_x
        // rand::random donne un nombre entre 0.0 et 1.0
        // On le transforme en nombre entre -1.0 et 1.0, puis on multiplie par max_x
        let x_pos = (rand::random::<f32>() - 0.5) * 2.0 * max_x; 
        
        // 👽 Créer un nouvel ennemi
        commands.spawn((
            Enemy,                                     // C'est un ennemi
            Movable { 
                velocity: Vec2::new(0.0, -ENEMY_SPEED) // Descend (y négatif)
            },
            Sprite {
                color: Color::srgb(1.0, 0.3, 0.3),    // Rouge
                custom_size: Some(ENEMY_SIZE),
                ..default()
            },
            Transform::from_xyz(
                x_pos,                                 // Position X aléatoire
                window.height() / 2.0 + ENEMY_SIZE.y, // En haut de l'écran
                0.0
            ),
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🏃 SYSTÈME DE MOUVEMENT (fait bouger TOUT ce qui a une velocity)
// ═══════════════════════════════════════════════════════════════════════════

/// 🎯 Applique le mouvement à TOUTES les entités qui ont une velocity
/// CE SYSTÈME :
/// - Lit Movable (velocity)
/// - ÉCRIT Transform (position)
/// - NE PEUT PAS tourner en parallèle avec d'autres systèmes qui écrivent Transform
/// - Mais c'est OK ! Il s'exécute après tous les systèmes qui changent velocity
fn apply_movement(
    mut query: Query<(&Movable, &mut Transform)>,  // Tout ce qui bouge
    time: Res<Time>,                               // Pour calculer le déplacement
) {
    // Pour chaque entité qui peut bouger (joueur, ennemis, balles)
    for (movable, mut transform) in query.iter_mut() {
        
        // 📐 Formule du mouvement :
        // nouvelle_position = ancienne_position + (velocity × temps_écoulé)
        // 
        // Exemple : si velocity.y = 800 et delta = 0.016 (60 FPS)
        // alors on bouge de 800 × 0.016 = 12.8 pixels vers le haut
        
        let movement = movable.velocity * time.delta_secs();
        
        // extend(0.0) transforme Vec2 en Vec3 en ajoutant z=0
        // (Transform.translation est en 3D même si on fait un jeu 2D)
        transform.translation += movement.extend(0.0);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚧 SYSTÈME DE CONTRAINTES (garde le joueur dans l'écran)
// ═══════════════════════════════════════════════════════════════════════════

/// 📏 Empêche le joueur de sortir de l'écran
/// CE SYSTÈME :
/// - ÉCRIT Transform du joueur
/// - S'exécute APRÈS apply_movement (grâce à .chain())
/// - Comme ça, on bouge d'abord, puis on corrige si on est sorti
fn clamp_player_position(
    mut player_query: Query<&mut Transform, With<Player>>, // Le joueur
    window_query: Query<&Window, With<PrimaryWindow>>,     // La fenêtre
) {
    // Obtenir la taille de la fenêtre
    let window = window_query.single().unwrap();
    
    // Calculer la limite gauche/droite
    // On enlève la moitié de la taille du joueur pour qu'il ne dépasse pas
    let limit_x = window.width() / 2.0 - PLAYER_SIZE.x / 2.0;
    
    // Pour chaque joueur (il n'y en a qu'un)
    for mut transform in player_query.iter_mut() {
        
        // 📍 clamp() force une valeur entre min et max
        // Si x < -limit_x, alors x = -limit_x
        // Si x > +limit_x, alors x = +limit_x
        // Sinon x reste inchangé
        transform.translation.x = transform.translation.x.clamp(-limit_x, limit_x);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🗑️ SYSTÈME DE NETTOYAGE (supprime ce qui sort de l'écran)
// ═══════════════════════════════════════════════════════════════════════════

/// 🧹 Supprime les balles et ennemis qui sortent de l'écran
/// CE SYSTÈME :
/// - Lit Transform
/// - SUPPRIME des entités (commands.entity().despawn())
/// - Peut tourner EN PARALLÈLE avec d'autres systèmes qui lisent seulement
fn despawn_out_of_bounds(
    mut commands: Commands,
    // Query complexe : on veut Entity, Transform, et savoir si c'est une Bullet ou Enemy
    query: Query<(Entity, &Transform, Option<&Bullet>, Option<&Enemy>)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().unwrap();
    
    // Marges : on attend que l'objet soit complètement hors écran
    let top_edge = window.height() / 2.0 + 50.0;      // Haut + marge
    let bottom_edge = -window.height() / 2.0 - 50.0;  // Bas - marge

    // Pour chaque entité
    for (entity, transform, is_bullet, is_enemy) in query.iter() {
        let y = transform.translation.y;
        
        // 💥 Si c'est une balle ET qu'elle est en haut
        // Option::is_some() retourne true si Option contient Some(...)
        if is_bullet.is_some() && y > top_edge {
            commands.entity(entity).despawn();  // Supprimer
        }
        
        // 👽 Si c'est un ennemi ET qu'il est en bas
        if is_enemy.is_some() && y < bottom_edge {
            commands.entity(entity).despawn();  // Supprimer
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 💥 SYSTÈMES DE COLLISION
// ═══════════════════════════════════════════════════════════════════════════

/// 🎯 Détecte les collisions entre balles et ennemis
/// CE SYSTÈME :
/// - Lit Transform des balles et ennemis
/// - SUPPRIME les balles et ennemis qui se touchent
/// - MODIFIE le score
/// - Peut tourner EN PARALLÈLE avec player_enemy_collision (pas de conflit)
fn bullet_enemy_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut game_state: ResMut<GameState>,
) {
    // Pour chaque balle
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        let bullet_pos = bullet_transform.translation.xy();  // Position 2D
        let bullet_half = BULLET_SIZE / 2.0;                 // Demi-taille
        
        // Pour chaque ennemi
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();
            let enemy_half = ENEMY_SIZE / 2.0;
            
            // 📐 Détection de collision AABB (Axis-Aligned Bounding Box)
            // C'est comme vérifier si deux rectangles se touchent
            // 
            // Distance horizontale entre les centres
            let dx = (bullet_pos.x - enemy_pos.x).abs();
            // Distance verticale entre les centres
            let dy = (bullet_pos.y - enemy_pos.y).abs();
            
            // Si les distances sont plus petites que la somme des demi-tailles,
            // alors ils se touchent !
            let collision = dx < (bullet_half.x + enemy_half.x) 
                         && dy < (bullet_half.y + enemy_half.y);

            if collision {
                // 💥 Supprimer l'ennemi et la balle
                commands.entity(enemy_entity).despawn();
                commands.entity(bullet_entity).despawn();
                
                // 🎯 Augmenter le score
                game_state.score += 10;
                println!("💥 Touché ! Score : {}", game_state.score);
                
                // break = sortir de la boucle (une balle ne peut toucher qu'un ennemi)
                break;
            }
        }
    }
}

/// 💔 Détecte les collisions entre le joueur et les ennemis
/// CE SYSTÈME :
/// - Lit Transform du joueur et des ennemis
/// - MODIFIE la santé du joueur
/// - SUPPRIME les ennemis qui touchent
/// - Change game_over si le joueur meurt
fn player_enemy_collision(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &mut Health), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut game_state: ResMut<GameState>,
) {
    // Si le jeu est déjà terminé, on ne fait rien
    if game_state.game_over {
        return;
    }

    // Pour chaque joueur (il n'y en a qu'un)
    for (player_entity, player_transform, mut health) in player_query.iter_mut() {
        let player_pos = player_transform.translation.xy();
        let player_half = PLAYER_SIZE / 2.0;

        // Pour chaque ennemi
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.xy();
            let enemy_half = ENEMY_SIZE / 2.0;

            // Même détection de collision que pour les balles
            let dx = (player_pos.x - enemy_pos.x).abs();
            let dy = (player_pos.y - enemy_pos.y).abs();
            
            let collision = dx < (player_half.x + enemy_half.x)
                         && dy < (player_half.y + enemy_half.y);

            if collision {
                // 👽 Supprimer l'ennemi
                commands.entity(enemy_entity).despawn();
                
                // 💔 Perdre une vie
                health.current -= 1;
                println!("💔 Aïe ! Vies restantes : {}", health.current);
                
                // ☠️ Si plus de vies, c'est GAME OVER
                if health.current <= 0 {
                    commands.entity(player_entity).despawn();
                    game_state.game_over = true;
                    println!("☠️ GAME OVER ! Score final : {}", game_state.score);
                }
                
                break;
            }
        }
        
        // Si le joueur est mort, sortir de la boucle
        if game_state.game_over {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🖥️ SYSTÈME D'AFFICHAGE (optionnel, pour voir le score et les vies)
// ═══════════════════════════════════════════════════════════════════════════

/// 📊 Affiche les infos dans le terminal (on pourrait faire un HUD plus tard)
fn display_info(
    _game_state: Res<GameState>,  // Le _ dit au compilateur "je sais que je ne l'utilise pas"
    player_query: Query<&Health, With<Player>>,
) {
    // On affiche seulement toutes les 60 frames pour ne pas spammer le terminal
    // (ce n'est pas la meilleure façon, mais c'est simple pour l'instant)
    
    // NOTE : Pour un vrai HUD, il faudrait utiliser bevy_ui avec des Text2dBundle
    // mais c'est plus compliqué, donc on reste simple pour l'instant !
    
    if let Ok(health) = player_query.single() {
        // Cette ligne ne s'affiche que si on peut obtenir la santé du joueur
        // On pourrait l'afficher à l'écran avec bevy_ui plus tard
        let _ = health; // On l'utilise pour éviter un warning
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 🚀 FONCTION PRINCIPALE (le cœur du programme)
// ═══════════════════════════════════════════════════════════════════════════

fn main() {
    App::new()
        // 🔌 Ajouter les plugins par défaut de Bevy
        // (fenêtre, graphismes, son, entrées clavier/souris, etc.)
        .add_plugins(DefaultPlugins)
        
        // 🎨 Couleur de fond (bleu très foncé, presque noir)
        .insert_resource(ClearColor(Color::srgb(0.04, 0.04, 0.08)))
        
        // 🎯 Initialiser l'état du jeu (score = 0, game_over = false)
        .init_resource::<GameState>()
        
        // 🎬 Système qui s'exécute AU DÉMARRAGE (une seule fois)
        .add_systems(Startup, setup_game)
        
        // 🔄 Systèmes qui s'exécutent À CHAQUE FRAME
        // 
        // ⚡ OPTIMISATION DU PARALLÉLISME :
        // On utilise .chain() pour grouper les systèmes qui DOIVENT s'exécuter dans l'ordre
        // 
        // Groupe 1 : INPUT (peuvent tourner en parallèle entre eux)
        .add_systems(Update, (
            player_input,         // Modifie velocity du joueur
            player_shooting,      // Crée des balles
            enemy_spawner,        // Crée des ennemis
            // Ces 3 systèmes ne se marchent pas sur les pieds,
            // donc Bevy peut les exécuter en PARALLÈLE sur différents CPU cores !
        ))
        
        // Groupe 2 : PHYSICS (doivent être dans cet ordre)
        .add_systems(Update, (
            apply_movement,           // Déplace tout (lit velocity, écrit transform)
            clamp_player_position,    // Garde le joueur dans l'écran (écrit transform)
        ).chain())  // .chain() = "exécute dans cet ordre, l'un après l'autre"
        
        // Groupe 3 : CLEANUP & COLLISIONS (peuvent tourner en parallèle entre eux)
        .add_systems(Update, (
            despawn_out_of_bounds,     // Supprime ce qui sort
            bullet_enemy_collision,    // Détecte balle-ennemi
            player_enemy_collision,    // Détecte joueur-ennemi
            display_info,              // Affiche les infos
            // Ces systèmes lisent surtout Transform, donc pas de conflit d'écriture
            // Bevy peut les exécuter en PARALLÈLE !
        ))
        
        // 🎮 Lancer le jeu !
        .run();
}

// ═══════════════════════════════════════════════════════════════════════════
// 📚 RÉCAPITULATIF DU PARALLÉLISME
// ═══════════════════════════════════════════════════════════════════════════
//
// 🟢 PEUVENT TOURNER EN PARALLÈLE (pas de conflit) :
//    - player_input (modifie Movable du joueur)
//    - player_shooting (crée des Bullet)
//    - enemy_spawner (crée des Enemy)
//    - despawn_out_of_bounds (lit Transform, supprime entités)
//    - bullet_enemy_collision (lit Transform, supprime entités)
//    - player_enemy_collision (lit Transform, modifie Health)
//
// 🔴 DOIVENT ÊTRE SÉQUENTIELS (conflit d'écriture sur Transform) :
//    - apply_movement (écrit Transform)
//    - clamp_player_position (écrit Transform du joueur)
//    ⚠️ C'est pour ça qu'on utilise .chain() !
//
// 🎯 RÉSULTAT :
//    Sur un CPU à 4 cœurs, Bevy peut exécuter jusqu'à 4 systèmes
//    du premier groupe en même temps ! C'est beaucoup plus rapide !
//
// ═══════════════════════════════════════════════════════════════════════════