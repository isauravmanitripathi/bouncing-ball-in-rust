use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::timer;
use nalgebra as na;
use mint;

const SCREEN_WIDTH: f32 = 1920.0;
const SCREEN_HEIGHT: f32 = 1080.0;
const BALL_RADIUS: f32 = 20.0;

struct Ball {
    position: na::Point2<f32>,
    velocity: na::Vector2<f32>,
}

impl Ball {
    fn new(x: f32, y: f32, vx: f32, vy: f32) -> Self {
        Ball {
            position: na::Point2::new(x, y),
            velocity: na::Vector2::new(vx, vy),
        }
    }

    fn update(&mut self) {
        self.position += self.velocity;

        if self.position.x <= BALL_RADIUS || self.position.x >= SCREEN_WIDTH - BALL_RADIUS {
            self.velocity.x = -self.velocity.x;
        }
        if self.position.y <= BALL_RADIUS || self.position.y >= SCREEN_HEIGHT - BALL_RADIUS {
            self.velocity.y = -self.velocity.y;
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult {
        let circle = Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            mint::Point2 { x: 0.0, y: 0.0 },
            BALL_RADIUS,
            0.1,
            Color::WHITE,
        )?;
        graphics::draw(ctx, &circle, DrawParam::default().dest(mint::Point2 { x: self.position.x, y: self.position.y }))
    }
}

struct MainState {
    ball: Ball,
}

impl MainState {
    fn new() -> GameResult<MainState> {
        let ball = Ball::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 5.0, 7.0);
        Ok(MainState { ball })
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.ball.update();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);
        self.ball.draw(ctx)?;
        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("bouncing_ball", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Bouncing Ball"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()
        .expect("Could not create ggez context!");

    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}
