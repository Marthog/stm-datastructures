#![feature(test)]
extern crate test;

extern crate stm;
extern crate stm_datastructures;

use stm::*;
use stm_datastructures::*;
use self::test::Bencher;
use std::sync::mpsc::{channel, sync_channel};
use std::thread;

#[inline(always)]
/// Run two threads `t1` and `t2`.
/// Wait for both to finish.
fn fork<T1, T2>(t1: T1, t2: T2)
where
    T1: FnOnce() -> () + Send + 'static,
    T2: FnOnce() -> (),
{
    let (sender, receiver) = channel();
    thread::spawn(move || {
        t1();
        // Signal that t1 is finished.
        sender.send(()).unwrap();
    });

    t2();
    // Wait for t1 to finish.
    receiver.recv().unwrap();
}

#[bench]
/// Send a bunch of values across a channel from std.
fn bench_std_channel(b: &mut Bencher) {

    b.iter(|| {
        let (sender, receiver) = channel();

        fork(
            move || for i in 0..1000 {
                assert_eq!(receiver.recv(), Ok(i));
            },
            || for i in 0..1000 {
                sender.send(i).unwrap();
            },
        );
    });
}

#[bench]
/// Send a bunch of values across a queue using stm.
fn bench_stm_queue(b: &mut Bencher) {
    b.iter(|| {
        let queue = Queue::new();
        // Make a clone for the second thread.
        let queue2 = queue.clone();

        fork(
            move || for i in 0..1000 {
                let x = atomically(|tx| queue.pop(tx));
                assert_eq!(x, i);
            },
            || for i in 0..1000 {
                atomically(|tx| queue2.push(tx, i));
            },
        );
    });
}


#[bench]
/// Send a bunch of values across a sync channel with size 1 from std.
///
/// The size of 1 forces the channel to hit the upper bound quickly and switch threads.
fn bench_std_sync_channel_1(b: &mut Bencher) {

    b.iter(|| {
        let (sender, receiver) = sync_channel(1);

        fork(
            move || for i in 0..1000 {
                assert_eq!(receiver.recv(), Ok(i));
            },
            || for i in 0..1000 {
                sender.send(i).unwrap();
            },
        );
    });
}

#[bench]
/// Send a bunch of values across a bounded queue with size 1 using stm.
///
/// The size of 1 forces the queue to hit the upper bound quickly and switch threads.
fn bench_stm_bqueue_1(b: &mut Bencher) {
    b.iter(|| {
        let queue = BoundedQueue::new(1);
        let queue2 = queue.clone();

        fork(
            move || for i in 0..1000 {
                let x = atomically(|tx| queue.pop(tx));
                assert_eq!(x, i);
            },
            || for i in 0..1000 {
                atomically(|tx| queue2.push(tx, i));
            },
        );
    });
}


#[bench]
/// Send a bunch of values across a sync channel with size 200 from std.
///
/// The size of 200 allows the channel to efficiently store a lot of
/// values in a ringbuffer before switching.
fn bench_std_sync_channel_200(b: &mut Bencher) {

    b.iter(|| {
        let (sender, receiver) = sync_channel(200);

        fork(
            move || for i in 0..1000 {
                assert_eq!(receiver.recv(), Ok(i));
            },
            || for i in 0..1000 {
                sender.send(i).unwrap();
            },
        );
    });
}

#[bench]
/// Send a bunch of values across a bounded queue with size 200 using stm.
///
/// The size of 200 allows the queue to store many values at once before switching threads.
fn bench_stm_bqueue_200(b: &mut Bencher) {
    b.iter(|| {
        let queue = BoundedQueue::new(200);
        let queue2 = queue.clone();

        fork(
            move || for i in 0..1000 {
                let x = atomically(|tx| queue.pop(tx));
                assert_eq!(x, i);
            },
            || for i in 0..1000 {
                atomically(|tx| queue2.push(tx, i));
            },
        );
    });
}
