mod turt;
mod svg;
use turt::Turtle;
use rand::random_range;
fn main() {
    for _ in 1..=100 {
        let turtle = Turtle::new();
        let _ = std::thread::spawn(move || {
            loop {
                let x = random_range(0.0..=30.0);
                let y = random_range(0.0..=100.0);
                let r = random_range(0.0..=1.0);
                let g = random_range(0.0..=1.0);
                let b = random_range(0.0..=1.0);
                let size_x = random_range(10.0..=200.0);
                let size_y = random_range(10.0..=200.0);
                turtle.left(x);
                turtle.forward(y);
                turtle.set_color(r, g, b, 1.0);
                turtle.set_size(size_x, size_y);
            }
        });
    }
    loop {
    }
}
