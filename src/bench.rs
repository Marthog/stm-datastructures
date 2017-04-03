extern crate test;

use stm::*;
use super::*;
use self::test::Bencher;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

#[bench]
fn bench_stm_channel(b: &mut Bencher) {
    
    b.iter(|| {
        let (sender, receiver) = channel();
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                receiver.recv();
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            sender.send(i);
        }
        let _ = receiver2.recv();
    });
}


#[bench]
fn bench_stm_sync_channel1(b: &mut Bencher) {
    
    b.iter(|| {
        let (sender, receiver) = sync_channel(1);
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                receiver.recv();
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            sender.send(i);
        }
        let _ = receiver2.recv();
    });
}


#[bench]
fn bench_stm_sync_channel200(b: &mut Bencher) {
    
    b.iter(|| {
        let (sender, receiver) = sync_channel(200);
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                receiver.recv();
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            sender.send(i);
        }
        let _ = receiver2.recv();
    });
}

#[bench]
fn bench_stm_queue(b: &mut Bencher) {
    b.iter(|| {
        let queue = Queue::new();
        let queue2 = queue.clone();
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                atomically(|trans| {
                    queue.pop(trans)
                });
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            atomically(|trans| {
                queue2.push(trans, i)
            });
        }
        let _ = receiver2.recv();
    });
}


#[bench]
fn bench_stm_bqueue1(b: &mut Bencher) {
    b.iter(|| {
        let queue = BoundedQueue::new(1);
        let queue2 = queue.clone();
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                atomically(|trans| {
                    queue.pop(trans)
                });
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            atomically(|trans| {
                queue2.push(trans, i)
            });
        }
        let _ = receiver2.recv();
    });
}

#[bench]
fn bench_stm_bqueue200(b: &mut Bencher) {
    b.iter(|| {
        let queue = BoundedQueue::new(200);
        let queue2 = queue.clone();
        let (sender2, receiver2) = channel();

        thread::spawn(move || {
            for i in 0..1000 {
                atomically(|trans| {
                    queue.pop(trans)
                });
            }
            let _ = sender2.send(());
        });

        for i in 0..1000 {
            atomically(|trans| {
                queue2.push(trans, i)
            });
        }
        let _ = receiver2.recv();
    });
}
