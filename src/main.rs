mod turt;
mod svg;
use turt::Turtle;
fn main() {
    let turtle = Turtle::new();
    let mut counter: u64 = 0;
    let _ = std::thread::spawn(move || {
        let turtle2 = Turtle::new();
        let mut counter_2: u64 = 0;
        loop {
            for i in 1..10 {
                turtle2.set_pos(20.0 * i as f32, 20.0 * i as f32);
                counter_2 = counter_2.wrapping_add(1);
                if counter_2 % 30 == 0 {
                    println!("loop2 {counter_2}");
                }
            }
        }
    });
    loop {
        for i in 1..10 {
            turtle.set_pos(20.0 * i as f32, 20.0 * i as f32);
            counter = counter.wrapping_add(1);
            if counter % 30 == 0 {
                println!("loop {counter}");
            }
        }
    }
}
