use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, MeshBuilder};
use nalgebra as na;
use mint;
use rand::Rng;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: f32 = 1080.0;
const SCREEN_HEIGHT: f32 = 1920.0;
const BALL_RADIUS: f32 = 20.0;
const COOLDOWN_DURATION: Duration = Duration::from_secs(1); // 1 second cooldown
const MAX_BALLS: usize = 1750; // Reduced maximum number of balls for performance

struct Ball {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
    color: Color,
    last_multiplied: Instant,
}

impl Ball {
    fn new(x: f32, y: f32, vx: f32, vy: f32, color: Color) -> Self {
        Ball {
            position: na::Point2::new(x, y),
            velocity: na::Vector2::new(vx, vy),
            color,
            last_multiplied: Instant::now(),
        }
    }

    fn update(&mut self) -> bool {
        self.position += self.velocity;
        let mut hit = false;

        if self.position.x - BALL_RADIUS <= 0.0 || self.position.x + BALL_RADIUS >= SCREEN_WIDTH {
            self.velocity.x = -self.velocity.x;
            self.position.x = self.position.x.max(BALL_RADIUS).min(SCREEN_WIDTH - BALL_RADIUS);
            hit = true;
        }
        if self.position.y - BALL_RADIUS <= 0.0 || self.position.y + BALL_RADIUS >= SCREEN_HEIGHT {
            self.velocity.y = -self.velocity.y;
            self.position.y = self.position.y.max(BALL_RADIUS).min(SCREEN_HEIGHT - BALL_RADIUS);
            hit = true;
        }

        hit
    }
}

struct MainState {
    balls: Vec<Ball>,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let initial_ball = Ball::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 5.0, 7.0, Color::WHITE);
        Ok(MainState { balls: vec![initial_ball] })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let mut rng = rand::thread_rng();
        let mut new_balls = vec![];

        let num_balls = self.balls.len(); // Get the current number of balls
        for ball in &mut self.balls {
            if ball.update() && ball.last_multiplied.elapsed() > COOLDOWN_DURATION && num_balls + new_balls.len() < MAX_BALLS {
                let color = Color::new(rng.gen(), rng.gen(), rng.gen(), 1.0);
                let angle: f32 = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let new_velocity = na::Vector2::new(angle.cos() * 5.0, angle.sin() * 5.0);
                new_balls.push(Ball::new(ball.position.x, ball.position.y, new_velocity.x, new_velocity.y, color));
                ball.last_multiplied = Instant::now();
            }
        }

        self.balls.append(&mut new_balls);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        let mut mesh_builder = MeshBuilder::new();
        for ball in &self.balls {
            mesh_builder.circle(
                graphics::DrawMode::fill(),
                mint::Point2 { x: ball.position.x, y: ball.position.y },
                BALL_RADIUS,
                0.1,
                ball.color,
            );
        }
        let mesh = mesh_builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("multiplying_balls", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Multiplying Balls"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()
        .expect("Could not create ggez context!");

    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
