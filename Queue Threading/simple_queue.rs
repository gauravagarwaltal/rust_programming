use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
extern crate queues;

use queues::*;
use std::str::FromStr;

pub fn parse<F: FromStr>(a: F) -> Result<F, <F>::Err> {
    std::result::Result::Ok(a)
}

fn queuing() {
    // Create a simple Queue
    let mut q: Queue<isize> = queue![];
    let mut five = Arc::new(queues::Queue::<isize>::new());

    for _ in 0..10 {
        let mut five = Arc::clone(&mut five);
        thread::spawn(move || {
            (*five.get()).add(1).unwrap();
            println!("{}", five.size());
        });
    }
    // // Add some elements to it
    // q.add(1).unwrap();
    // q.add(-2).unwrap();
    // q.add(3).unwrap();

    // // Check the Queue's size
    // let size = q.size(); // 3
    // println!("{}", size);

    // // Remove an element
    // if size != 0 {
    //     q.remove().unwrap(); // Ok(1)
    // }
    // // Check the Queue's size
    // let size = q.size(); // 2

    // // Peek at the next element scheduled for removal
    // if size != 0 {
    //     println!("{}", q.peek().unwrap());
    // }
    // // Ok(-2)

    // // Confirm that the Queue size hasn't changed
    // let size = q.size(); // 2

    // // Remove the remaining elements
    // if size != 0 {
    //     q.remove().unwrap(); // Ok(1)
    // }

    // let size = q.size(); // 2

    // // Remove the remaining elements
    // if size != 0 {
    //     q.remove().unwrap(); // Ok(3)
    // }

    // let size = q.size();
    // // Peek into an empty Queue
    // if size != 0 {
    //     q.peek().unwrap(); // Raises an error
    // }
    // // Attempt to remove an element from an empty Queue
    // if size != 0 {
    //     q.remove().unwrap(); // Raises an error
    // }
}
fn main() {
    queuing();
    let (tx, rx) = mpsc::channel();
    // let val = Arc::new(AtomicUsize::new(5));

    // for _ in 0..10 {
    //     let val = Arc::clone(&val);

    //     thread::spawn(move || {
    //         let v = val.fetch_add(1, Ordering::SeqCst);
    //         println!("{:?}", v);
    //     });
    // }
    // let tx1 = mpsc::Sender::clone(&tx);
    // thread::spawn(move || {
    //     let vals = vec![
    //         String::from("hi"),
    //         String::from("from"),
    //         String::from("the"),
    //         String::from("thread"),
    //     ];

    //     for val in vals {
    //         tx1.send(val).unwrap();
    //         thread::sleep(Duration::from_millis(10));
    //     }
    // });

    // thread::spawn(move || {
    //     let vals = vec![
    //         String::from("more"),
    //         String::from("messages"),
    //         String::from("for"),
    //         String::from("you"),
    //     ];

    //     for val in vals {
    //         tx.send(val).unwrap();
    //         thread::sleep(Duration::from_millis(1));
    //     }
    // });

    // for received in rx {
    //     println!("Got: {}", received);
    // }
}
