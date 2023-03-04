use std::thread;

fn main() {
    let a = thread::spawn(|| println!("thread a"));
    let b = thread::spawn(|| println!("thread b"));
    a.join().unwrap();
    b.join().unwrap();
}
