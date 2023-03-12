use zombie::Serialize;

#[derive(Serialize)]
struct S {
    #[id(1)]
    #[pbtype(sint32)]
    x: i32,
}

fn main() {
    println!("hello world");

    let mut v = Vec::new();

    let s = S { x: 3 };
    s.serialize(&mut v).unwrap();

    println!("{:?}", v);
}
