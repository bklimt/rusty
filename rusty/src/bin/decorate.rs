use zombie::Serialize;

#[derive(Serialize)]
struct S {
    #[id(1)]
    x: i32,
}

fn main() {
    println!("hello world");

    let s = S { x: 3 };
    s.serialize();
}
