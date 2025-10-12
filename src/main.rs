mod turt;
use turt::Turtle;
fn main() {
    let turtle = Turtle::new();
    for i in 1..10 {
        turtle.set_pos(20.0 * i as f32, 20.0 * i as f32);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    dbg!(&turtle);
}
