// Cargo.toml:
// [package]
// name = "xgalaga_rust"
// version = "0.1.0"
// edition = "2021"

// [dependencies]
// macroquad = "0.4"
// rand = "0.8"

///////////////////////////////////////////////////////////////////////


// Importation des modules nécessaires
// macroquad : framework de jeu 2D pour Rust
// rand : bibliothèque pour la génération de nombres aléatoires
use macroquad::prelude::*;
use ::rand::Rng;

// ============================================================================
// CONSTANTES DE CONFIGURATION DU JEU
// ============================================================================

// Dimensions de la fenêtre de jeu
const SCREEN_WIDTH: f32 = 1200.0;
const SCREEN_HEIGHT: f32 = 700.0;

// Configuration du joueur
const PLAYER_SIZE: f32 = 40.0;      // Taille du joueur (diamètre du triangle)
const PLAYER_SPEED: f32 = 5.0;      // Vitesse de déplacement du joueur

// Configuration des projectiles
const BULLET_SIZE: f32 = 8.0;       // Taille des projectiles
const BULLET_SPEED: f32 = 8.0;      // Vitesse des projectiles du joueur

// Configuration des ennemis
const ENEMY_SIZE: f32 = 35.0;       // Taille des ennemis
const ENEMY_SPEED: f32 = 1.5;       // Vitesse de base des ennemis
const ENEMY_BULLET_SPEED: f32 = 4.0; // Vitesse des projectiles ennemis

// ============================================================================
// STRUCTURE DU JOUEUR
// ============================================================================

// Le joueur est défini avec l'attribut Clone pour permettre la copie si nécessaire
#[derive(Clone)]
struct Player {
    pos: Vec2,    // Position (x, y) du joueur
    lives: i32,   // Nombre de vies restantes
    score: i32,   // Score actuel
}

impl Player {
    // Constructeur pour créer un nouveau joueur
    fn new() -> Self {
        Self {
            // Position initiale : centre horizontal, près du bas de l'écran
            pos: Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - 80.0),
            lives: 3,  // 3 vies au départ
            score: 0,  // Score initial à 0
        }
    }

    // Met à jour l'état du joueur (appelé chaque frame)
    fn update(&mut self) {
        // Déplacement vers la gauche (flèche gauche ou touche A)
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.pos.x -= PLAYER_SPEED;
        }
        // Déplacement vers la droite (flèche droite ou touche D)
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.pos.x += PLAYER_SPEED;
        }

        // Empêcher le joueur de sortir de l'écran
        // clamp() assure que la position reste dans les limites
        self.pos.x = self.pos.x.clamp(PLAYER_SIZE / 2.0, SCREEN_WIDTH - PLAYER_SIZE / 2.0);
    }

    // Dessine le joueur à l'écran
    fn draw(&self) {
        // Dessine le vaisseau sous forme de triangle (pointe vers le haut)
        draw_triangle(
            // Sommet supérieur (pointe du vaisseau)
            Vec2::new(self.pos.x, self.pos.y - PLAYER_SIZE / 2.0),
            // Sommet inférieur gauche
            Vec2::new(self.pos.x - PLAYER_SIZE / 2.0, self.pos.y + PLAYER_SIZE / 2.0),
            // Sommet inférieur droit
            Vec2::new(self.pos.x + PLAYER_SIZE / 2.0, self.pos.y + PLAYER_SIZE / 2.0),
            SKYBLUE,  // Couleur principale du vaisseau
        );
        
        // Dessine le cockpit (cercle au centre)
        draw_circle(self.pos.x, self.pos.y, 8.0, BLUE);
    }
}

// ============================================================================
// STRUCTURE DES PROJECTILES
// ============================================================================

struct Bullet {
    pos: Vec2,              // Position actuelle
    velocity: Vec2,         // Vitesse et direction
    is_player_bullet: bool, // True si c'est un projectile du joueur
}

impl Bullet {
    // Crée un nouveau projectile du joueur
    fn new_player(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            // Se déplace vers le haut (y négatif)
            velocity: Vec2::new(0.0, -BULLET_SPEED),
            is_player_bullet: true,
        }
    }

    // Crée un nouveau projectile ennemi
    fn new_enemy(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            // Se déplace vers le bas (y positif)
            velocity: Vec2::new(0.0, ENEMY_BULLET_SPEED),
            is_player_bullet: false,
        }
    }

    // Met à jour la position du projectile
    fn update(&mut self) {
        self.pos += self.velocity;
    }

    // Vérifie si le projectile est sorti de l'écran
    fn is_off_screen(&self) -> bool {
        self.pos.y < 0.0 || self.pos.y > SCREEN_HEIGHT
    }

    // Dessine le projectile
    fn draw(&self) {
        // Les projectiles du joueur sont jaunes, ceux des ennemis sont rouges
        let color = if self.is_player_bullet { YELLOW } else { RED };
        draw_circle(self.pos.x, self.pos.y, BULLET_SIZE / 2.0, color);
    }
}

// ============================================================================
// STRUCTURE DES ENNEMIS
// ============================================================================

#[derive(Clone)]
struct Enemy {
    pos: Vec2,              // Position actuelle
    velocity: Vec2,         // Vitesse et direction
    movement_pattern: i32,  // Type de mouvement (0, 1, ou autre)
    time: f32,              // Temps écoulé (pour les mouvements sinusoïdaux)
}

impl Enemy {
    // Crée un nouvel ennemi
    fn new(x: f32, y: f32, pattern: i32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            velocity: Vec2::new(ENEMY_SPEED, ENEMY_SPEED),
            movement_pattern: pattern,
            time: 0.0,
        }
    }

    // Met à jour la position de l'ennemi
    fn update(&mut self, dt: f32) {
        // Incrémente le temps (pour les animations)
        self.time += dt;

        // Applique le mouvement selon le pattern
        match self.movement_pattern {
            0 => {
                // Pattern 0 : Mouvement horizontal simple
                self.pos.x += self.velocity.x;
                // Rebond sur les bords gauche et droit
                if self.pos.x < 50.0 || self.pos.x > SCREEN_WIDTH - 50.0 {
                    self.velocity.x *= -1.0;  // Inverse la direction horizontale
                    self.pos.y += 20.0;       // Descend un peu
                }
            }
            1 => {
                // Pattern 1 : Mouvement sinusoïdal
                self.pos.y += ENEMY_SPEED * 0.5;  // Descente lente
                // Mouvement horizontal en sinus
                self.pos.x += (self.time * 3.0).sin() * 2.0;
            }
            _ => {
                // Pattern par défaut : Mouvement diagonal
                self.pos += self.velocity;
                // Rebond sur les bords gauche et droit
                if self.pos.x < 50.0 || self.pos.x > SCREEN_WIDTH - 50.0 {
                    self.velocity.x *= -1.0;
                }
            }
        }
    }

    // Dessine l'ennemi à l'écran
    fn draw(&self) {
        // Corps principal de l'alien (deux cercles concentriques)
        draw_circle(self.pos.x, self.pos.y, ENEMY_SIZE / 2.0, DARKGREEN);
        draw_circle(self.pos.x, self.pos.y, ENEMY_SIZE / 3.0, GREEN);
        
        // Yeux de l'alien (deux cercles rouges)
        draw_circle(self.pos.x - 8.0, self.pos.y - 5.0, 5.0, RED);
        draw_circle(self.pos.x + 8.0, self.pos.y - 5.0, 5.0, RED);
        
        // Antennes de l'alien (deux lignes avec des cercles au bout)
        draw_line(self.pos.x - 10.0, self.pos.y - 15.0, 
                  self.pos.x - 10.0, self.pos.y - 25.0, 2.0, GREEN);
        draw_line(self.pos.x + 10.0, self.pos.y - 15.0, 
                  self.pos.x + 10.0, self.pos.y - 25.0, 2.0, GREEN);
        
        // Extrémités des antennes (cercles jaunes)
        draw_circle(self.pos.x - 10.0, self.pos.y - 25.0, 3.0, YELLOW);
        draw_circle(self.pos.x + 10.0, self.pos.y - 25.0, 3.0, YELLOW);
    }

    // Détermine si l'ennemi doit tirer (aléatoirement)
    fn should_shoot(&self) -> bool {
        // 1 chance sur 200 par frame (0.5% de probabilité)
        ::rand::thread_rng().gen_ratio(1, 200)
    }
}

// ============================================================================
// ÉTAT PRINCIPAL DU JEU
// ============================================================================

struct GameState {
    player: Player,           // Instance du joueur
    bullets: Vec<Bullet>,     // Liste de tous les projectiles en jeu
    enemies: Vec<Enemy>,      // Liste de tous les ennemis en jeu
    enemy_spawn_timer: f32,   // Timer pour le spawn des ennemis
    game_over: bool,          // True si le jeu est terminé
    paused: bool,             // True si le jeu est en pause
    wave: i32,                // Numéro de la vague actuelle
}

impl GameState {
    // Crée un nouvel état de jeu
    fn new() -> Self {
        Self {
            player: Player::new(),
            bullets: Vec::new(),
            enemies: Vec::new(),
            enemy_spawn_timer: 0.0,
            game_over: false,
            paused: false,
            wave: 1,  // Commence à la vague 1
        }
    }

    // Génère une nouvelle vague d'ennemis
    fn spawn_wave(&mut self) {
        // Le nombre d'ennemis augmente avec la vague
        let enemies_per_wave = 5 + (self.wave * 2);
        
        // Crée les ennemis avec des positions et patterns variés
        for i in 0..enemies_per_wave {
            // Position X : répartie sur la largeur de l'écran
            let x = 100.0 + (i as f32 * 100.0) % (SCREEN_WIDTH - 200.0);
            // Position Y : organisée en rangées
            let y = 50.0 + ((i / 8) as f32 * 60.0);
            // Pattern aléatoire entre 0, 1 et 2
            let pattern = ::rand::thread_rng().gen_range(0..3);
            self.enemies.push(Enemy::new(x, y, pattern));
        }
    }

    // Met à jour l'état du jeu (logique principale)
    fn update(&mut self, dt: f32) {
        // Si le jeu est terminé ou en pause, on ne fait rien
        if self.game_over || self.paused {
            return;
        }

        // Met à jour la position du joueur
        self.player.update();

        // Gestion du tir du joueur (touche ESPACE)
        if is_key_pressed(KeyCode::Space) {
            // Crée un nouveau projectile à la position du joueur
            self.bullets.push(Bullet::new_player(self.player.pos.x, self.player.pos.y));
        }

        // Met à jour tous les projectiles et supprime ceux hors écran
        self.bullets.retain_mut(|bullet| {
            bullet.update();
            !bullet.is_off_screen()  // Garde seulement les projectiles à l'écran
        });

        // Met à jour tous les ennemis
        for enemy in &mut self.enemies {
            enemy.update(dt);
            
            // Certains ennemis tirent aléatoirement
            if enemy.should_shoot() {
                self.bullets.push(Bullet::new_enemy(enemy.pos.x, enemy.pos.y));
            }
        }

        // Si tous les ennemis sont détruits, on passe à la vague suivante
        if self.enemies.is_empty() {
            self.wave += 1;
            self.spawn_wave();
        }

        // Vérifie les collisions entre les différents éléments
        self.check_collisions();

        // Vérifie si le jeu est terminé (plus de vies)
        if self.player.lives <= 0 {
            self.game_over = true;
        }
    }

    // Gère toutes les collisions du jeu
    fn check_collisions(&mut self) {
        // Listes des index à supprimer
        let mut bullets_to_remove = Vec::new();
        let mut enemies_to_remove = Vec::new();

        // ====================================================================
        // COLLISIONS : PROJECTILES JOUEUR vs ENNEMIS
        // ====================================================================
        for (b_idx, bullet) in self.bullets.iter().enumerate() {
            // On ne traite que les projectiles du joueur
            if !bullet.is_player_bullet {
                continue;
            }

            // Vérifie la collision avec chaque ennemi
            for (e_idx, enemy) in self.enemies.iter().enumerate() {
                // Calcule la distance entre le projectile et l'ennemi
                let dist = (bullet.pos - enemy.pos).length();
                // Si la distance est inférieure à la somme des rayons, il y a collision
                if dist < (BULLET_SIZE + ENEMY_SIZE) / 2.0 {
                    bullets_to_remove.push(b_idx);
                    enemies_to_remove.push(e_idx);
                    self.player.score += 100;  // Incrémente le score
                }
            }
        }

        // ====================================================================
        // COLLISIONS : PROJECTILES ENNEMIS vs JOUEUR
        // ====================================================================
        for (b_idx, bullet) in self.bullets.iter().enumerate() {
            // On ne traite que les projectiles ennemis
            if bullet.is_player_bullet {
                continue;
            }

            // Vérifie la collision avec le joueur
            let dist = (bullet.pos - self.player.pos).length();
            if dist < (BULLET_SIZE + PLAYER_SIZE) / 2.0 {
                bullets_to_remove.push(b_idx);
                self.player.lives -= 1;  // Enlève une vie
                // Replace le joueur à sa position initiale
                self.player.pos = Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - 80.0);
            }
        }

        // ====================================================================
        // COLLISIONS : ENNEMIS vs JOUEUR (collision directe)
        // ====================================================================
        for (e_idx, enemy) in self.enemies.iter().enumerate() {
            let dist = (enemy.pos - self.player.pos).length();
            if dist < (ENEMY_SIZE + PLAYER_SIZE) / 2.0 {
                enemies_to_remove.push(e_idx);
                self.player.lives -= 1;  // Enlève une vie
                // Replace le joueur à sa position initiale
                self.player.pos = Vec2::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT - 80.0);
            }
        }

        // ====================================================================
        // SUPPRESSION DES ÉLÉMENTS COLLIDÉS
        // ====================================================================
        
        // Nettoie les listes d'index pour éviter les doublons
        bullets_to_remove.sort_unstable();
        bullets_to_remove.dedup();
        
        // Supprime les projectiles (en commençant par la fin pour éviter les décalages)
        for &idx in bullets_to_remove.iter().rev() {
            if idx < self.bullets.len() {
                self.bullets.remove(idx);
            }
        }

        // Même procédure pour les ennemis
        enemies_to_remove.sort_unstable();
        enemies_to_remove.dedup();
        
        for &idx in enemies_to_remove.iter().rev() {
            if idx < self.enemies.len() {
                self.enemies.remove(idx);
            }
        }
    }

    // Dessine l'ensemble du jeu à l'écran
    fn draw(&self) {
        // Efface l'écran avec une couleur noire
        clear_background(BLACK);

        // ====================================================================
        // DESSIN DU FOND (ÉTOILES)
        // ====================================================================
        for i in 0..100 {
            // Position pseudo-aléatoire des étoiles (utilise un modulo)
            let x = (i * 73) % (SCREEN_WIDTH as i32);
            let y = (i * 47) % (SCREEN_HEIGHT as i32);
            draw_circle(x as f32, y as f32, 1.5, WHITE);
        }

        // ====================================================================
        // DESSIN DES OBJETS DE JEU
        // ====================================================================
        
        // Dessine le joueur
        self.player.draw();

        // Dessine tous les projectiles
        for bullet in &self.bullets {
            bullet.draw();
        }

        // Dessine tous les ennemis
        for enemy in &self.enemies {
            enemy.draw();
        }

        // ====================================================================
        // HUD (HEADS-UP DISPLAY) - INFORMATIONS DE JEU
        // ====================================================================
        
        // Score du joueur
        draw_text(
            &format!("Score: {}", self.player.score),
            10.0, 30.0, 30.0, WHITE,
        );
        
        // Nombre de vies restantes
        draw_text(
            &format!("Vies: {}", self.player.lives),
            10.0, 60.0, 30.0, WHITE,
        );
        
        // Numéro de la vague actuelle
        draw_text(
            &format!("Vague: {}", self.wave),
            10.0, 90.0, 30.0, WHITE,
        );

        // ====================================================================
        // MESSAGES CONTEXTUELS
        // ====================================================================
        
        // Message de pause
        if self.paused {
            let text = "PAUSE - P pour reprendre";
            // Calcule les dimensions du texte pour le centrer
            let dims = measure_text(text, None, 40, 1.0);
            draw_text(
                text,
                SCREEN_WIDTH / 2.0 - dims.width / 2.0,  // Centre horizontal
                SCREEN_HEIGHT / 2.0,                    // Centre vertical
                40.0, YELLOW,
            );
        }

        // Message de Game Over
        if self.game_over {
            let text = "GAME OVER";
            let dims = measure_text(text, None, 60, 1.0);
            draw_text(
                text,
                SCREEN_WIDTH / 2.0 - dims.width / 2.0,
                SCREEN_HEIGHT / 2.0 - 30.0,  // Légèrement au-dessus du centre
                60.0, RED,
            );
            
            // Instructions pour recommencer
            let restart_text = "Appuyez sur R pour recommencer";
            let dims2 = measure_text(restart_text, None, 30, 1.0);
            draw_text(
                restart_text,
                SCREEN_WIDTH / 2.0 - dims2.width / 2.0,
                SCREEN_HEIGHT / 2.0 + 40.0,  // Légèrement en dessous du centre
                30.0, WHITE,
            );
        }

        // ====================================================================
        // INSTRUCTIONS PERMANENTES
        // ====================================================================
        draw_text("Flèches/WASD: Bouger | ESPACE: Tirer | P: Pause", 
                  10.0, SCREEN_HEIGHT - 10.0, 20.0, GRAY);
    }
}

// ============================================================================
// CONFIGURATION DE LA FENÊTRE
// ============================================================================

fn window_conf() -> Conf {
    Conf {
        window_title: "XGalaga Rust".to_string(),  // Titre de la fenêtre
        window_width: SCREEN_WIDTH as i32,         // Largeur de la fenêtre
        window_height: SCREEN_HEIGHT as i32,       // Hauteur de la fenêtre
        window_resizable: false,                   // Fenêtre non redimensionnable
        ..Default::default()                       // Valeurs par défaut pour les autres options
    }
}

// ============================================================================
// FONCTION PRINCIPALE
// ============================================================================

#[macroquad::main(window_conf)]  // Configure et lance la fenêtre macroquad
async fn main() {
    // Initialise l'état du jeu
    let mut game = GameState::new();
    // Génère la première vague d'ennemis
    game.spawn_wave();

    // Boucle principale du jeu (exécutée à chaque frame)
    loop {
        // Récupère le temps écoulé depuis la dernière frame (delta time)
        let dt = get_frame_time();

        // ====================================================================
        // GESTION DES TOUCHES GLOBALES
        // ====================================================================
        
        // Touche P : Met en pause/reprend le jeu
        if is_key_pressed(KeyCode::P) {
            game.paused = !game.paused;
        }

        // Touche R : Recommence le jeu (seulement en Game Over)
        if game.game_over && is_key_pressed(KeyCode::R) {
            game = GameState::new();
            game.spawn_wave();
        }

        // Touche Échap : Quitte le jeu
        if is_key_pressed(KeyCode::Escape) {
            break;  // Sort de la boucle principale
        }

        // ====================================================================
        // MISE À JOUR ET AFFICHAGE DU JEU
        // ====================================================================
        
        // Met à jour la logique du jeu
        game.update(dt);
        // Dessine tout à l'écran
        game.draw();

        // Attend la frame suivante (nécessaire pour le rendu asynchrone)
        next_frame().await;
    }
}