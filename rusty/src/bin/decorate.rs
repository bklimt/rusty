use zombie::Serialize;

#[derive(Serialize)]
struct S {
    #[id]
    x: i32,
}

fn main() {
    println!("hello world");

    let s = S { x: 3 };
    s.serialize();
}
