use std::sync::{Mutex, Arc,mpsc};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

const count_thread: usize = 10;

fn transmit(d: usize, tx: mpsc::Sender<usize>){
    thread::spawn(move || {
        tx.send(d).unwrap();
        println!("hehe {}", d);
    });
}

fn threading_on_queue(){
    let t_queue = Arc::new(Mutex::new(VecDeque::<u32>::new()));
    let mut thread_handler = vec![];

    for i in 0..10 {
        let c = Arc::clone(&t_queue);
        let thread_spawn = thread::spawn(move || {
            let mut temp = c.lock().unwrap();
            temp.push(1);
        });
        thread_handler.push(thread_spawn);
    }
    for handler in thread_handler{
        handler.join().unwrap();
    }
    println!("Resultted queue : {:?}", *t_queue.lock().unwrap());
}

fn main(){
    threading_on_queue();
    let (tx, rx) = mpsc::channel();
    for i in 0..count_thread{
        transmit(i, tx.clone());
    }

    for v in rx.iter().take(count_thread){
        println!("{}:....", v);
    }
}