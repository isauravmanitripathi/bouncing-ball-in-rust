use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::timer;
use nalgebra as na;
use mint;
use rand::Rng;

const SCREEN_WIDTH: f32 = 1080.0;
const SCREEN_HEIGHT: f32 = 1920.0;
const BALL_RADIUS: f32 = 20.0;

struct Ball {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
    color: Color,
}

impl Ball {
    fn new(x: f32, y: f32, vx: f32, vy: f32, color: Color) -> Self {
        Ball {
            position: na::Point2::new(x, y),
            velocity: na::Vector2::new(vx, vy),
            color,
        }
    }

    fn update(&mut self) -> bool {
        self.position += self.velocity;
        let mut hit = false;

        if self.position.x <= BALL_RADIUS || self.position.x >= SCREEN_WIDTH - BALL_RADIUS {
            self.velocity.x = -self.velocity.x;
            hit = true;
        }
        if self.position.y <= BALL_RADIUS || self.position.y >= SCREEN_HEIGHT - BALL_RADIUS {
            self.velocity.y = -self.velocity.y;
            hit = true;
        }

        hit
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            mint::Point2 { x: 0.0, y: 0.0 },
            BALL_RADIUS,
            0.1,
            self.color,
        )?;
        graphics::draw(ctx, &circle, DrawParam::default().dest(mint::Point2 { x: self.position.x, y: self.position.y }))
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
        let mut new_ball = None;

        for ball in &mut self.balls {
            if ball.update() {
                let color = Color::new(rng.gen(), rng.gen(), rng.gen(), 1.0);
                let angle: f32 = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let new_velocity = na::Vector2::new(angle.cos() * 5.0, angle.sin() * 5.0);
                new_ball = Some(Ball::new(ball.position.x, ball.position.y, new_velocity.x, new_velocity.y, color));
                break; // Add only one new ball per update cycle
            }
        }

        if let Some(ball) = new_ball {
            self.balls.push(ball);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);
        for ball in &self.balls {
            ball.draw(ctx)?;
        }
        graphics::present(ctx)?;
        timer::yield_now();
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
