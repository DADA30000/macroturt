use std::sync::{Arc, Mutex, OnceLock, mpsc};
use std::thread;
use macroquad::prelude::*;

struct TurtleInfo {
    x: f32,
    y: f32,
    rotation: f32
}

#[derive(Debug)]
pub struct Turtle {
    id: usize
}

type RenderJob = Box<dyn FnOnce() + Send>;

static CHANNELS: OnceLock<(
    mpsc::Sender<RenderJob>,
    Mutex<mpsc::Receiver<RenderJob>>
)> = OnceLock::new();

static TURTLES: OnceLock<Arc<Mutex<Vec<TurtleInfo>>>> = OnceLock::new();

static RENDER: OnceLock<thread::JoinHandle<()>> = OnceLock::new();

fn spawn_render() -> &'static thread::JoinHandle<()> {
    RENDER.get_or_init(|| {
        let turtles_handle = turtles().clone();
        thread::spawn(move || {
            macroquad::Window::new("The Final, Correct Solution", async move {
                loop {
                    clear_background(RED);
                    let turtles_handle_async = turtles_handle.clone();
                    let turtles_lock = turtles_handle_async.lock().unwrap();
                    for turt in &*turtles_lock {
                        draw_line(turt.x, turt.y, turt.x + 100.0, turt.y + 100.0, 10.0, BLACK);
                        draw_fps();
                    }
                    next_frame().await;
                }
            });
        })
    })
}

fn turtles() -> &'static Arc<Mutex<Vec<TurtleInfo>>> {
    TURTLES.get_or_init(|| {
        Arc::new(Mutex::new(Vec::new()))
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
        let _ = spawn_render();
        Turtle { id: new_id }
    }
    pub fn set_pos(&self, x: f32, y: f32) {
        let turtle = &mut turtles().lock().unwrap()[self.id];
        turtle.x = x;
        turtle.y = y;
    }
}
