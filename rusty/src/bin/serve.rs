extern crate rusty;

use serde::Serialize;
use warp::Filter;

#[derive(Serialize)]
struct S {
    i: i32,
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
