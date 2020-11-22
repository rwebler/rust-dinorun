#![warn(clippy::all, clippy::pedantic)]
use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    Pause,
    End,
}

const SCREEN_WIDTH: i32 = 80;
//const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 35.0;
const FLOOR: i32 = 40;
const PLAYER_COLUMN: i32 = 10;

struct Player {
    x: i32,
    y: i32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }
    fn render(&mut self, ctx: &mut BTerm, color: (u8, u8, u8)) {
        ctx.set(
            PLAYER_COLUMN,
            self.y,
            RGB::named(YELLOW),
            color,
            to_cp437('@'),
        );
    }
    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 && self.y < FLOOR {
            self.velocity += 0.8;
        }
        self.y += self.velocity as i32;
        self.x += 1;
        if self.y > FLOOR {
            self.y = FLOOR;
            self.velocity = 0.0;
        }
    }
    fn jump(&mut self) {
        self.velocity = -4.0;
    }
}

struct Obstacle {
    x: i32,
    y: i32,
    velocity: f32,
    symbol: char,
    color: (u8, u8, u8),
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        let x_diff = random.range(0, SCREEN_WIDTH / 2);
        let mut height = random.range(FLOOR-5, FLOOR+1);
        let mut velocity = random.range(-1.5, score as f32 * 0.02);
        if velocity < 0.0 {
            velocity = 0.0;
            height = FLOOR;
        }
        Obstacle {
            x: x + x_diff,
            y: height,
            velocity,
            symbol: if velocity > 0.0 { '<' } else {'!'},
            color: if velocity > 0.0 { BROWN1 } else { GREEN },
        }
    }
    fn render(&mut self, ctx: &mut BTerm, player_x: i32, color: (u8, u8, u8)) {
        self.x -= self.velocity as i32;
        let screen_x = self.x - player_x;
        ctx.set(
            screen_x,
            self.y,
            self.color,
            color,
            to_cp437(self.symbol),
        );
    }
    fn hit_obstacle(&mut self, player: &Player) -> bool {
        player.x == (self.x - PLAYER_COLUMN) && player.y == self.y
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacles: Vec<Obstacle>,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        let mut obstacles = Vec::new();
        obstacles.push(Obstacle::new(SCREEN_WIDTH, 0));
        State {
            player: Player::new(PLAYER_COLUMN, FLOOR),
            frame_time: 0.0,
            obstacles,
            mode: GameMode::Menu,
            score: 0,
        }
    }
    fn sky(&mut self) -> (u8, u8, u8) {
        let modscore = self.score % 50;
        match modscore {
            0..=10 => CORNFLOWERBLUE,
            11..=20 => MEDIUM_BLUE,
            21..=30 => DARK_BLUE,
            31..=40 => MIDNIGHT_BLUE,
            _ => SLATE_BLUE,
        }
    }
    fn play(&mut self, ctx: &mut BTerm) {
        let color = self.sky();
        ctx.cls_bg(color);
        for i in 0..SCREEN_WIDTH {
            ctx.set(
                i,
                FLOOR+1,
                RGB::named(GREEN),
                color,
                to_cp437('-'),
            );
        }
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.mode = GameMode::Pause,
                VirtualKeyCode::Space => {
                    if self.player.y == FLOOR {
                        self.player.jump();
                    }
                },
                _ => {}
            }
        }
        self.player.render(ctx, color);
        let len = self.obstacles.len();
        for obstacle in &mut self.obstacles {
            obstacle.render(ctx, self.player.x, color);
            if obstacle.hit_obstacle(&self.player) {
                self.mode = GameMode::End;
            }
        }
        let diff = self.player.x - 5;
        self.obstacles.retain(|o| o.x > diff);
        let newlen = self.obstacles.len();
        let newscore = len - newlen;
        self.score += newscore as i32;

        if (self.obstacles[newlen - 1].x - self.player.x) < (SCREEN_WIDTH * 9 / 10) {
            self.obstacles.push(Obstacle::new(self.player.x + SCREEN_WIDTH, self.score));
        }

        ctx.print(0, 0, "Press SPACE to jump.");
        ctx.print(0, 1, &format!("Score: {}", self.score));
        ctx.print(0, 2, &format!("Obstacles: {}", newlen));
    }

    fn restart(&mut self) {
        self.player = Player::new(PLAYER_COLUMN, FLOOR);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        let mut obstacles = Vec::new();
        obstacles.push(Obstacle::new(SCREEN_WIDTH, 0));
        self.obstacles = obstacles;
        self.score = 0;
    }
    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Dinorun");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");
        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
    fn pause(&mut self, ctx: &mut BTerm) {
        self.dead(ctx);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::Pause => self.pause(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::simple80x50()
        .with_title("Dinorun")
        .build()?;
    main_loop(context, State::new())
}
