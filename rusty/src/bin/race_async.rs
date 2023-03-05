use rand::random;
use std::{io::Result, mem::swap, time::Duration};
use tokio::{
    spawn,
    sync::mpsc::{channel, Sender},
    time::sleep,
};

async fn print_after(i: usize, after: Duration, sender: Sender<usize>) {
    sleep(after).await;
    println!("sending {i}");
    if let Err(err) = sender.send(i).await {
        println!("send error: {}", err);
    }
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

    let (sender, mut receiver) = channel::<usize>(1);

    let mut handles = Vec::new();
    for (i, r) in v.iter().enumerate() {
        let sender = sender.clone();
        let duration = Duration::from_secs(*r as u64);
        println!("thread {i} will finish after {} seconds.", r);
        let handle = spawn(async move { print_after(i, duration, sender).await });
        handles.push(handle);
    }
    drop(sender);

    while let Some(i) = receiver.recv().await {
        println!("received {}.", i);
    }

    for handle in handles {
        handle.await?;
    }

    Ok(())
}
