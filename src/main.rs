use macroquad::prelude::*;
pub use turt::svg::svg_to_texture;

fn window_conf() -> Conf {
    Conf {
        window_title: "ahuet".to_owned(),
        platform: miniquad::conf::Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    //let mut degree: f32 = 0.0;
    let mut velocity: (f32,f32) = (0.0,0.0); // x,y
    let mut cords: (f32,f32) = (200.0,200.0); // x,y
    const MOVE_SPEED: f32 = 5000.0;
    //const ROTATION_SPEED: f32 = 4000.0;
    const FRICTION: f32 = 0.1;
    let cursor_texture = svg_to_texture(&std::fs::read_to_string("cursor.svg").unwrap_or(String::from("<svg width=\"8000\" height=\"8000\" viewBox=\"0 0 8000 8000\" fill=\"none\" xmlns=\"http://www.w3.org/2000/svg\"><path d=\"M 4000 1000 L 2000 7000 L 3500 5500 L 4500 5500 L 6000 7000 Z\" fill=\"white\"/></svg>")));
    let desired_x: f32 = 1000.0;
    let desired_y: f32 = 100.0;
    loop {
        let (mouse_x, mouse_y) = mouse_position();
        let delta_x = mouse_x - cords.0;
        let delta_y = mouse_y - cords.1;
        let angle = delta_y.atan2(delta_x);
        let delta = get_frame_time();
        let keys = get_keys_down();
        let rotation = angle + std::f32::consts::PI / 2.0;
        if keys.contains(&KeyCode::W) || get_keys_down().contains(&KeyCode::Up) {
            velocity.1 -= MOVE_SPEED * delta;
            // velocity.2 -= ROTATION_SPEED * delta;
        }
        if keys.contains(&KeyCode::S) || get_keys_down().contains(&KeyCode::Down){
            velocity.1 += MOVE_SPEED * delta;
            // velocity.2 += ROTATION_SPEED * delta;
        }
        if keys.contains(&KeyCode::D) || get_keys_down().contains(&KeyCode::Right) {
            velocity.0 += MOVE_SPEED * delta;
            // velocity.2 += ROTATION_SPEED * delta;
        }
        if keys.contains(&KeyCode::A) || get_keys_down().contains(&KeyCode::Left) {
            velocity.0 -= MOVE_SPEED * delta;
            // velocity.2 -= ROTATION_SPEED * delta;
        }
        cords.0 += velocity.0 * delta;
        cords.1 += velocity.1 * delta;
        //degree += velocity.2 * delta;
        clear_background(BLACK);
        draw_texture_ex(
            &cursor_texture,
            cords.0 - desired_x / 2.0, cords.1 - desired_y / 2.0, YELLOW, DrawTextureParams { rotation, dest_size: Some(Vec2 { x: desired_x, y: desired_y }), ..Default::default()});
        let mut keys = get_keys_down().iter().map(|x| format!("{:?}",x)).collect::<String>();
        keys.push_str(&format!(" FPS: {} X: {} Y: {} Vel X: {} Vel Y {} Res {}x{}",get_fps(), cords.0, cords.1, velocity.0, velocity.1, screen_width(), screen_height()));
        draw_text(&keys, 20.0, 20.0, 30.0, DARKGRAY);
        velocity.0 *= FRICTION.powf(delta);
        velocity.1 *= FRICTION.powf(delta);
        //velocity.2 *= FRICTION.powf(delta);
        next_frame().await
    }
}
