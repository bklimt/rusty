use std::{
    mem::swap,
    sync::mpsc::{channel, Sender},
    thread::{sleep, spawn},
    time::Duration,
};

use rand::random;

fn print_after(i: usize, after: Duration, sender: Sender<usize>) {
    sleep(after);
    println!("sending {i}");
    if let Err(err) = sender.send(i) {
        println!("error sending: {}", err);
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

fn main() {
    let n = 10;
    let mut v = (1..=n).collect::<Vec<i32>>();
    shuffle(&mut v);

    let (sender, receiver) = channel::<usize>();

    let mut handles = Vec::new();
    for (i, r) in v.iter().enumerate() {
        let duration = Duration::from_secs(*r as u64);
        println!("thread {i} will finish after {} seconds.", r);
        let sender = sender.clone();
        let handle = spawn(move || print_after(i, duration, sender));
        handles.push(handle);
    }
    drop(sender);

    while let Ok(i) = receiver.recv() {
        println!("received {i}");
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
