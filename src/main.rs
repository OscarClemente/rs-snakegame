extern crate sdl2;

use core::ops::Add;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::video::Window;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum GameState {
    Playing,
    Paused,
    Failed,
    Won,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PlayerDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Point(pub i32, pub i32);

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

const GRID_X_SIZE: i32 = 40;
const GRID_Y_SIZE: i32 = 30;
const DOT_SIZE_IN_PXS: i32 = 20;

pub struct GameContext {
    pub player_position: Vec<Point>,
    pub player_direction: PlayerDirection,
    pub food: Point,
    pub state: GameState,
}

impl GameContext {
    pub fn new() -> GameContext {
        GameContext {
            player_position: vec![Point(3, 1), Point(2, 1), Point(1, 1)],
            player_direction: PlayerDirection::Right,
            state: GameState::Paused,
            food: random_point(),
        }
    }
    pub fn next_tick(&mut self) {
        if self.state != GameState::Playing {
            return;
        }
        let head_position = self.player_position.first().unwrap();
        let mut next_head_position = match self.player_direction {
            PlayerDirection::Up => *head_position + Point(0, -1),
            PlayerDirection::Down => *head_position + Point(0, 1),
            PlayerDirection::Right => *head_position + Point(1, 0),
            PlayerDirection::Left => *head_position + Point(-1, 0),
        };

        next_head_position = self.position_wrapping(next_head_position);
        if self.player_position.contains(&next_head_position) {
            self.state = GameState::Failed;
            return;
        }

        if next_head_position != self.food {
            self.player_position.pop();
        }
        self.player_position.reverse();
        self.player_position.push(next_head_position);
        self.player_position.reverse();

        if next_head_position == self.food {
            self.regenerate_food();
        }
    }

    fn position_wrapping(&mut self, next_head_position: Point) -> Point {
        match next_head_position {
            Point(GRID_X_SIZE, y) => Point(0, y),
            Point(-1, y) => Point(GRID_X_SIZE - 1, y),
            Point(x, GRID_Y_SIZE) => Point(x, 0),
            Point(x, -1) => Point(x, GRID_Y_SIZE - 1),
            _ => next_head_position,
        }
    }

    pub fn move_up(&mut self) {
        if self.player_direction != PlayerDirection::Down {
            self.player_direction = PlayerDirection::Up;
        }
    }

    pub fn move_down(&mut self) {
        if self.player_direction != PlayerDirection::Up {
            self.player_direction = PlayerDirection::Down;
        }
    }

    pub fn move_right(&mut self) {
        if self.player_direction != PlayerDirection::Left {
            self.player_direction = PlayerDirection::Right;
        }
    }

    pub fn move_left(&mut self) {
        if self.player_direction != PlayerDirection::Right {
            self.player_direction = PlayerDirection::Left;
        }
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            state => state,
        }
    }

    pub fn regenerate_food(&mut self) {
        let mut new_food = random_point();
        while self.player_position.contains(&new_food) {
            new_food = random_point();
        }
        self.food = new_food;
    }
}

fn random_point() -> Point {
    let mut rng = rand::thread_rng();
    Point(rng.gen_range(0..GRID_X_SIZE), rng.gen_range(0..GRID_Y_SIZE))
}

pub struct Renderer {
    canvas: WindowCanvas,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, String> {
        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(Renderer { canvas })
    }

    fn draw_dot(&mut self, point: &Point) -> Result<(), String> {
        let Point(x, y) = point;
        self.canvas.fill_rect(Rect::new(
            x * DOT_SIZE_IN_PXS as i32,
            y * DOT_SIZE_IN_PXS as i32,
            DOT_SIZE_IN_PXS.try_into().unwrap(),
            DOT_SIZE_IN_PXS.try_into().unwrap(),
        ))?;

        Ok(())
    }

    pub fn draw(&mut self, context: &GameContext) -> Result<(), String> {
        self.draw_background(context);
        self.draw_player(context)?;
        self.draw_food(context)?;
        self.canvas.present();

        Ok(())
    }

    fn draw_background(&mut self, context: &GameContext) {
        let color = match context.state {
            GameState::Playing => Color::RGB(0, 0, 0),
            GameState::Paused => Color::RGB(30, 30, 30),
            GameState::Won => Color::RGB(30, 200, 30),
            GameState::Failed => Color::RGB(200, 30, 30),
        };
        self.canvas.set_draw_color(color);
        self.canvas.clear();
    }

    fn draw_player(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::GREEN);
        for point in &context.player_position {
            self.draw_dot(point)?;
        }

        Ok(())
    }

    fn draw_food(&mut self, context: &GameContext) -> Result<(), String> {
        self.canvas.set_draw_color(Color::RED);
        self.draw_dot(&context.food)?;
        Ok(())
    }
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Snake Game",
            (GRID_X_SIZE * DOT_SIZE_IN_PXS).try_into().unwrap(),
            (GRID_Y_SIZE * DOT_SIZE_IN_PXS).try_into().unwrap(),
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut context = GameContext::new();
    let mut renderer = Renderer::new(window)?;
    renderer.draw(&context)?;
    let mut frame_counter = 0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::W | Keycode::K => context.move_up(),
                    Keycode::A | Keycode::H => context.move_left(),
                    Keycode::S | Keycode::J => context.move_down(),
                    Keycode::D | Keycode::L => context.move_right(),
                    Keycode::Escape => context.toggle_pause(),
                    _ => {}
                },
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 100));

        frame_counter += 1;
        if frame_counter % 10 == 0 {
            context.next_tick();
            frame_counter = 0;
        }

        renderer.draw(&context)?;
    }

    Ok(())
}
