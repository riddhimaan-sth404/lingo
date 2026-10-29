//! Prototype F: Concurrency Isolation Prototype
//! Demonstrates message passing and Send/Sync isolation boundaries to prevent data races.

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

pub struct Message<T> {
    pub payload: T,
}

pub fn spawn_worker<T: Send + 'static>(receiver: Receiver<Message<T>>, mut handler: impl FnMut(T) + Send + 'static) {
    thread::spawn(move || {
        while let Ok(msg) = receiver.recv() {
            handler(msg.payload);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isolated_concurrency() {
        let (tx, rx): (Sender<Message<i64>>, Receiver<Message<i64>>) = channel();
        let (done_tx, done_rx) = channel();

        spawn_worker(rx, move |val| {
            assert_eq!(val, 999);
            done_tx.send(true).unwrap();
        });

        tx.send(Message { payload: 999 }).unwrap();
        assert!(done_rx.recv().unwrap());
    }
}
