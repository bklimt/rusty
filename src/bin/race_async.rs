use rand::random;
use std::{io::Result, mem::swap, time::Duration};
use tokio::{spawn, time::sleep};

async fn print_after(i: usize, after: Duration) {
    sleep(after).await;
    println!("hello {i}");
}

fn shuffle(v: &mut Vec<i32>) {
    for i in 0..v.len() {
        let j = i + random::<usize>() % (v.len() - i);
        if i != j {
            let (vi, vj) = v.split_at_mut(j);
            swap(vi.get_mut(i).unwrap(), vj.get_mut(0).unwrap());
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let n = 10;
    let mut v = (1..=n).collect::<Vec<i32>>();
    shuffle(&mut v);

    let mut handles = Vec::new();
    for (i, r) in v.iter().enumerate() {
        let duration = Duration::from_secs(*r as u64);
        println!("thread {i} will finish after {} seconds.", r);
        let handle = spawn(async move { print_after(i, duration).await });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
