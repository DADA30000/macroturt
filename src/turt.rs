use std::sync::{Mutex, OnceLock, mpsc};
use macroquad::prelude::*;

struct TurtleInfo {
    x: f32,
    y: f32,
    rotation: f32
}

struct Turtle {
    id: usize
}

type RenderJob = Box<dyn FnOnce() + Send>;

static CHANNELS: OnceLock<(
    mpsc::Sender<RenderJob>,
    Mutex<mpsc::Receiver<RenderJob>>
)> = OnceLock::new();

static TURTLES: OnceLock<Mutex<Vec<TurtleInfo>>> = OnceLock::new();

fn turtles() -> &'static Mutex<Vec<TurtleInfo>> {
    TURTLES.get_or_init(|| {
        Mutex::new(Vec::new())
    })
}

fn channels() -> &'static (mpsc::Sender<RenderJob>,Mutex<mpsc::Receiver<RenderJob>>) {
    CHANNELS.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        (tx, Mutex::new(rx))
    })
}

impl Turtle {
    pub fn new() -> Turtle {
        let mut turtles_vec = turtles().lock().unwrap();
        let new_id = turtles_vec.len();
        let info = TurtleInfo {
            x: 0.0,
            y: 0.0,
            rotation: 0.0
        };
        turtles_vec.push(info);
        Turtle { id: new_id }
    }
    pub async fn set_pos(&self, x: f32, y: f32) {
        let turtle = &mut turtles().lock().unwrap()[self.id];
        turtle.x = x;
        turtle.y = y;
        next_frame().await;
    }
}
