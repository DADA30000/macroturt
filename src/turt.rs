use std::sync::{Arc, Mutex, Condvar, OnceLock, RwLock};
use std::thread;
use macroquad::prelude::camera::mouse::Camera;
use macroquad::prelude::*;
use std::collections::VecDeque;
use crate::svg::svg_to_texture;
use num_traits::AsPrimitive;
use std::f32::consts::{FRAC_PI_2, TAU};

#[derive(Clone)]
struct TurtleInfo {
    x: f32,
    y: f32,
    rotation: f32,
    visual_rotation_offset: f32,
    texture: &'static Texture2D,
    state: VecDeque<Arc<dyn Fn() + Send + Sync>>,
    color: Color,
    max_hist: usize,
    size: Vec2
}

#[derive(Debug)]
pub struct Turtle {
    id: usize
}

type WrappedTurtleInfo = Arc<Mutex<Arc<RwLock<Vec<TurtleInfo>>>>>;

static CONDVAR: OnceLock<Arc<(Mutex<bool>,Condvar)>> = OnceLock::new();
static MIN_FRAMETIME: OnceLock<Arc<RwLock<f64>>> = OnceLock::new();
static TURTLES: OnceLock<WrappedTurtleInfo> = OnceLock::new();
static DEFAULT_TEXTURE: OnceLock<Texture2D> = OnceLock::new();
static RENDER: OnceLock<thread::JoinHandle<()>> = OnceLock::new();

fn get_min_frametime() -> &'static Arc<RwLock<f64>> {
    MIN_FRAMETIME.get_or_init(|| {
        Arc::new(RwLock::new(1.0_f64 / 1000.0_f64))
    })
}

fn spawn_render() -> &'static thread::JoinHandle<()> {
    RENDER.get_or_init(|| {
        let condvar_handle = condvar().clone();
        let condvar_handle_graceful = condvar().clone();
        let turtles_clone = turtles().lock().unwrap().clone();
        let min_frametime_clone = get_min_frametime().clone();
        thread::spawn(move || {
            let window_conf = macroquad::conf::Conf {
                miniquad_conf: miniquad::conf::Conf {
                    platform: miniquad::conf::Platform {
                        swap_interval: Some(0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            };
            macroquad::Window::from_config(window_conf, async move {
                let turtles_handle = turtles_clone.clone();
                let mut prev_time: f64 = 0.0;
                loop {
                    set_camera(&Camera2D {
                        zoom: vec2(1.0 / screen_width() * 2.0, 1.0 / screen_height() * 2.0),
                        ..Default::default()
                    });
                    let min_frametime = *min_frametime_clone.read().unwrap();
                    let min_time_to_pass = prev_time + min_frametime;
                    let delta = ((min_time_to_pass - get_time()) * 1000000.0) as u64;
                    if min_time_to_pass < get_time() {
                        prev_time = get_time();
                        clear_background(WHITE);
                        default_texture(); // Any Texture2D init needs to be in render loop
                        let turtles_lock = turtles_handle.read().unwrap();
                        let turtles_clone = turtles_lock.clone();
                        std::mem::drop(turtles_lock);
                        for turt in turtles_clone {
                            for command in turt.state {
                                command();
                            }
                            draw_texture_ex(turt.texture, turt.x - turt.size.x / 2.0, turt.y - turt.size.y / 2.0, turt.color, DrawTextureParams { 
                                rotation: turt.visual_rotation_offset + turt.rotation, 
                                dest_size: Some(turt.size), 
                                ..Default::default() 
                            });
                            
                        }
                        let (lock, cvar) = &*condvar_handle;
                        let mut finished = lock.lock().unwrap();
                        *finished = true;
                        cvar.notify_all();
                        next_frame().await;
                    } else {
                        thread::sleep(std::time::Duration::from_nanos(delta));
                    }
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
        svg_to_texture("<svg width=\"800\" height=\"800\" viewBox=\"0 0 800 800\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M 400 100 L 200 700 L 350 550 L 450 550 L 600 700 Z\" fill=\"white\"/></svg>")
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
    pub fn set_max_fps(fps: impl AsPrimitive<f64>) {
        let fps = fps.as_();
        let frametime = 1.0 / fps;
        *get_min_frametime().write().unwrap() = frametime;
    }
    fn change_turtle_properties(turtle_id: &Turtle, task: impl FnOnce(&mut TurtleInfo)) {
        let turtles_mutex_lock = turtles().lock().unwrap();
        let mut turtles_write_lock = turtles_mutex_lock.write().unwrap();
        let turtle = turtles_write_lock.get_mut(turtle_id.id).unwrap();
        task(turtle);
        std::mem::drop(turtles_write_lock);
        std::mem::drop(turtles_mutex_lock);
        Self::wait_for_render();
    }
    fn wait_for_render() {
        let (lock, cvar) = &**condvar();
        let mut finished = lock.lock().unwrap();
        while !*finished {
            finished = cvar.wait(finished).unwrap();
        }
        *finished = false;
    }
    pub fn set_color(&self, r: impl AsPrimitive<f32>, g: impl AsPrimitive<f32>, b: impl AsPrimitive<f32>, a: impl AsPrimitive<f32>) {
        let (r, g, b, a) = (r.as_(), g.as_(), b.as_(), a.as_());
        Self::change_turtle_properties(self, move |turtle| {
            turtle.color = Color { r, g, b, a };
        });
    }
    // use only by using Self::change_turtle_properties()
    fn push_state(turtle: &mut TurtleInfo, state: impl Fn() + Send + Sync + 'static) {
        turtle.state.truncate(turtle.max_hist - 1);
        turtle.state.push_back(Arc::new(state));
    }
    pub fn new() -> Turtle {
        let _ = spawn_render();
        Self::wait_for_render();
        
        let turtles_mutex_lock = turtles().lock().unwrap();
        let mut turtles_write_lock = turtles_mutex_lock.write().unwrap();
        let new_id = turtles_write_lock.len();
        let info = TurtleInfo {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            visual_rotation_offset: FRAC_PI_2, // to rotate texture 90 degrees
            texture: default_texture(),
            state: VecDeque::new(),
            color: BLACK,
            max_hist: 10,
            size: Vec2 { x: 48.0, y: 48.0 }
        };
        turtles_write_lock.push(info);
        std::mem::drop(turtles_write_lock);
        std::mem::drop(turtles_mutex_lock);

        Turtle { id: new_id }
    }
    pub fn max_hist(&self, new_limit: impl AsPrimitive<usize>) {
        let new_limit = new_limit.as_();
        Self::change_turtle_properties(self, move |turtle| {
            turtle.max_hist = new_limit;
        });
    }
    pub fn set_pos(&self, x: impl AsPrimitive<f32>, y: impl AsPrimitive<f32>) {
        Self::change_turtle_properties(self, move |turtle| {
            let (x, y) = (x.as_(), y.as_());
            let (prev_x, prev_y, color) = (turtle.x, turtle.y, turtle.color);
            turtle.x = x;
            turtle.y = y;
            Self::push_state(turtle, move || draw_line(x, y, prev_x, prev_y, 1.0, color));
        });
    }
    pub fn set_size(&self, x: impl AsPrimitive<f32>, y: impl AsPrimitive<f32>) {
        Self::change_turtle_properties(self, move |turtle| {
            let (x, y) = (x.as_(), y.as_());
            turtle.size.x = x;
            turtle.size.y = y;
        });
    }
    pub fn forward(&self, distance: impl AsPrimitive<f32>) {
        let distance = distance.as_();
        Self::change_turtle_properties(self, move |turtle| {
            let (x, y) = (turtle.x, turtle.y);
            turtle.x += distance * turtle.rotation.cos();
            turtle.y += distance * turtle.rotation.sin();
            let (new_x, new_y, color) = (turtle.x, turtle.y, turtle.color);
            Self::push_state(turtle, move || draw_line(x, y, new_x, new_y, 1.0, color));
        });
    }
    pub fn left(&self, degrees: impl AsPrimitive<f32>) {
        let degrees = degrees.as_();
        Self::change_turtle_properties(self, move |turtle| {
            turtle.rotation = ((turtle.rotation - degrees.to_radians()) % TAU + TAU) % TAU;
        });
    }
    pub fn right(&self, degrees: impl AsPrimitive<f32>) {
        let degrees = degrees.as_();
        Self::change_turtle_properties(self, move |turtle| {
            turtle.rotation = ((turtle.rotation + degrees.to_radians()) % TAU + TAU) % TAU;
        });
    }
    //pub fn forward(&self, distance: f32) {
    //    
    //}
}
