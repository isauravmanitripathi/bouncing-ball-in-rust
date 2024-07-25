use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam, MeshBuilder};
use nalgebra as na;
use mint;
use rand::Rng;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use image::{ImageBuffer, Rgba};
use crossbeam::channel::{unbounded, Sender};
use sha2::{Sha256, Digest};

const SCREEN_WIDTH: f32 = 1080.0;
const SCREEN_HEIGHT: f32 = 1920.0;
const BALL_RADIUS: f32 = 20.0;
const COOLDOWN_DURATION: Duration = Duration::from_secs(1); // 1 second cooldown
const MAX_BALLS: usize = 300; // Reduced maximum number of balls for performance
const FRAMES_PER_SECOND: u32 = 30;
const RECORD_DURATION: f32 = 65.0; // 65 seconds
const OUTPUT_FOLDER: &str = "/Volumes/hard-drive/animation-generation/bouncing_ball/video-files";

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
    frame_count: u32,
    recording: bool,
    temp_dir: PathBuf,
    screenshot_sender: Sender<(Vec<u8>, PathBuf)>,
}

impl MainState {
    fn new(screenshot_sender: Sender<(Vec<u8>, PathBuf)>) -> GameResult<MainState> {
        let initial_ball = Ball::new(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0, 5.0, 7.0, Color::WHITE);
        let mut temp_dir = std::env::current_dir().unwrap();
        temp_dir.push("temp_frames");
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
        println!("Temporary directory created at: {:?}", temp_dir);
        Ok(MainState { balls: vec![initial_ball], frame_count: 0, recording: true, temp_dir, screenshot_sender })
    }

    fn save_screenshot(&self, ctx: &mut Context, path: &PathBuf) -> GameResult {
        let screenshot = graphics::screenshot(ctx)?;
        let width = screenshot.width() as u32;
        let height = screenshot.height() as u32;
        let image_data = screenshot.to_rgba8(ctx)?;

        println!("Expected image data size: {}", width * height * 4);
        println!("Actual image data size: {}", image_data.len());

        if image_data.len() != (width * height * 4) as usize {
            println!("Image data size mismatch: expected {}, got {}", width * height * 4, image_data.len());
            return Err(ggez::GameError::RenderError(String::from("Image data size mismatch")));
        }

        // Calculate and print the checksum
        let checksum = Sha256::digest(&image_data);
        println!("Screenshot data checksum: {:?}", checksum);

        println!("Sending screenshot data to thread...");
        self.screenshot_sender.send((image_data, path.clone())).expect("Failed to send screenshot data");
        Ok(())
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
            )?;
        }
        let mesh = mesh_builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;

        graphics::present(ctx)?;

        if self.recording {
            let filename = self.temp_dir.join(format!("frame_{:05}.png", self.frame_count));
            println!("Saving screenshot to: {:?}", filename);

            // Save screenshot using the new method
            self.save_screenshot(ctx, &filename)?;

            println!("Screenshot saved to: {:?}", filename);
            self.frame_count += 1;

            if self.frame_count >= FRAMES_PER_SECOND * RECORD_DURATION as u32 {
                self.recording = false;
                println!("Recording complete. {} frames saved.", self.frame_count);
                println!("Frames saved in temporary directory: {:?}", self.temp_dir);

                // Create the output folder if it doesn't exist
                fs::create_dir_all(OUTPUT_FOLDER)?;

                // Run ffmpeg to create the video
                let output_video = format!("{}/output.mp4", OUTPUT_FOLDER);
                let status = Command::new("ffmpeg")
                    .args(&[
                        "-framerate", &FRAMES_PER_SECOND.to_string(),
                        "-i", &format!("{}/frame_%05d.png", self.temp_dir.display()),
                        "-c:v", "libx264",
                        "-pix_fmt", "yuv420p",
                        &output_video
                    ])
                    .status()
                    .expect("Failed to execute ffmpeg");
                if status.success() {
                    println!("Video saved to {}", output_video);
                } else {
                    println!("Failed to create video.");
                }

                // Clean up the temporary directory
                fs::remove_dir_all(&self.temp_dir).expect("Failed to delete temp directory");
                println!("Temporary directory deleted: {:?}", self.temp_dir);
            }
        }

        Ok(())
    }
}

fn main() -> GameResult {
    // Create a channel for sending screenshot data
    let (screenshot_sender, screenshot_receiver) = unbounded();

    // Spawn a thread to handle saving screenshots
    std::thread::spawn(move || {
        while let Ok((image_data, path)) = screenshot_receiver.recv() {
            let width = 1080;  // SCREEN_WIDTH
            let height = 1920; // SCREEN_HEIGHT
            println!("Saving screenshot to disk: {:?}", path);

            // Validate checksum before creating ImageBuffer
            let received_checksum = Sha256::digest(&image_data);
            println!("Received screenshot data checksum: {:?}", received_checksum);

            match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width, height, image_data) {
                Some(image) => {
                    if let Err(e) = image.save(&path) {
                        eprintln!("Failed to save image: {:?}", e);
                    } else {
                        println!("Successfully saved image: {:?}", path);
                    }
                }
                None => {
                    eprintln!("Failed to create ImageBuffer from raw data");
                }
            }
        }
    });

    let (ctx, event_loop) = ContextBuilder::new("ball_video", "Author")
        .window_setup(ggez::conf::WindowSetup::default().title("Ball Video"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT))
        .build()
        .expect("Could not create ggez context!");

    let state = MainState::new(screenshot_sender)?;
    event::run(ctx, event_loop, state)
}
