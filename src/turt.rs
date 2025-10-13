use std::sync::{Arc, Mutex, Condvar, OnceLock, RwLock};
use std::thread;
use macroquad::prelude::*;
use crate::svg::svg_to_texture;

#[derive(Clone)]
struct TurtleInfo {
    x: f32,
    y: f32,
    rotation: f32,
    texture: &'static Texture2D
}

#[derive(Debug)]
pub struct Turtle {
    id: usize
}

type WrappedTurtleInfo = Arc<Mutex<Arc<RwLock<Vec<TurtleInfo>>>>>;

static CONDVAR: OnceLock<Arc<(Mutex<bool>,Condvar)>> = OnceLock::new();
static TURTLES: OnceLock<WrappedTurtleInfo> = OnceLock::new();
static DEFAULT_TEXTURE: OnceLock<Texture2D> = OnceLock::new();
static RENDER: OnceLock<thread::JoinHandle<()>> = OnceLock::new();

fn spawn_render() -> &'static thread::JoinHandle<()> {
    RENDER.get_or_init(|| {
        let condvar_handle = condvar().clone();
        let condvar_handle_graceful = condvar().clone();
        let turtles_clone = turtles().lock().unwrap().clone();
        thread::spawn(move || {
            let window_conf = macroquad::conf::Conf {
                miniquad_conf: miniquad::conf::Conf {
                    platform: miniquad::conf::Platform {
                        swap_interval: Some(1),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            macroquad::Window::from_config(window_conf, async move {
                let turtles_handle = turtles_clone.clone();
                loop {
                    clear_background(RED);
                    default_texture(); // Any Texture2D init needs to be in render loop
                    let turtles_lock = turtles_handle.read().unwrap();
                    let turtles_clone = turtles_lock.clone();
                    std::mem::drop(turtles_lock);
                    for turt in turtles_clone {
                        draw_texture_ex(turt.texture, turt.x, turt.y, YELLOW, DrawTextureParams { 
                            rotation: turt.rotation, 
                            dest_size: Some(Vec2 {x: 64.0, y: 64.0}), 
                            ..Default::default() 
                        });
                        draw_fps();
                    }
                    let (lock, cvar) = &*condvar_handle;
                    let mut finished = lock.lock().unwrap();
                    *finished = true;
                    cvar.notify_all();
                    next_frame().await;
                }
            });
            loop {
                let (lock, cvar) = &*condvar_handle_graceful;
                let mut finished = lock.lock().unwrap();
                *finished = true;
                cvar.notify_all();
            };
        })
    })
}

fn default_texture() -> &'static Texture2D {
    DEFAULT_TEXTURE.get_or_init(|| {
        svg_to_texture("<svg width=\"8000\" height=\"8000\" viewBox=\"0 0 8000 8000\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M 4000 1000 L 2000 7000 L 3500 5500 L 4500 5500 L 6000 7000 Z\" fill=\"white\"/></svg>")
    })
}

fn turtles() -> &'static WrappedTurtleInfo {
    TURTLES.get_or_init(|| {
        Arc::new(Mutex::new(Arc::new(RwLock::new(Vec::new()))))
    })
}

fn condvar() -> &'static Arc<(Mutex<bool>,Condvar)> {
    CONDVAR.get_or_init(|| {
        Arc::new((Mutex::new(false), Condvar::new()))
    })
}
impl Turtle {
    fn change_turtle_properties(turtle_id: &Turtle, task: impl FnOnce(&mut TurtleInfo)) {
        let turtles_mutex_lock = turtles().lock().unwrap();
        let mut turtles_write_lock = turtles_mutex_lock.write().unwrap();
        if let Some(turtle) = turtles_write_lock.get_mut(turtle_id.id) {
            task(turtle);
        };
        std::mem::drop(turtles_write_lock);
        std::mem::drop(turtles_mutex_lock);
        Self::wait_for_render();
    }
    //fn send_render_command() {}
    //fn send_additional_state() {}
    fn wait_for_render() {
        let (lock, cvar) = &**condvar();
        let mut finished = lock.lock().unwrap();
        while !*finished {
            finished = cvar.wait(finished).unwrap();
        }
        *finished = false;
    }
    pub fn new() -> Turtle {
        let _ = spawn_render();
        Self::wait_for_render();

        let info = TurtleInfo {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            texture: default_texture()
        };

        let turtles_mutex_lock = turtles().lock().unwrap();
        let mut turtles_write_lock = turtles_mutex_lock.write().unwrap();
        let new_id = turtles_write_lock.len();
        turtles_write_lock.push(info);
        std::mem::drop(turtles_write_lock);
        std::mem::drop(turtles_mutex_lock);

        Self::wait_for_render();

        Turtle { id: new_id }
    }
    pub fn set_pos(&self, x: f32, y: f32) {
        Self::change_turtle_properties(self, move |turtle| {
            turtle.x = x;
            turtle.y = y;
        });
    }
    //pub fn forward(&self, distance: f32) {
    //    
    //}
}
