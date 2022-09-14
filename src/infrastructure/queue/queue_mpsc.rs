use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, SendError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

use super::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct QueueMPSC<E> {
    rx: Arc<Mutex<Receiver<E>>>,
    tx: Sender<E>,
    count: Arc<Mutex<QueueSize>>,
    capacity: Arc<Mutex<QueueSize>>,
}

impl<E: Element + 'static> QueueBehavior<E> for QueueMPSC<E> {
    fn len(&self) -> QueueSize {
        let count_guard = self.count.lock().unwrap();
        count_guard.clone()
    }

    fn capacity(&self) -> QueueSize {
        let capacity_guard = self.capacity.lock().unwrap();
        capacity_guard.clone()
    }

    fn offer(&mut self, e: E) -> anyhow::Result<()> {
        match self.tx.send(e) {
            Ok(_) => {
                let mut count_guard = self.count.lock().unwrap();
                count_guard.increment();
                Ok(())
            }
            Err(SendError(e)) => Err(anyhow::Error::new(QueueError::OfferError(e))),
        }
    }

    fn poll(&mut self) -> Result<Option<E>> {
        let receiver_guard = self.rx.lock().unwrap();
        match receiver_guard.try_recv() {
            Ok(e) => {
                let mut count_guard = self.count.lock().unwrap();
                count_guard.decrement();
                Ok(Some(e))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(anyhow::Error::new(QueueError::<E>::PoolError)),
        }
    }
}

impl<E: Element + 'static> QueueMPSC<E> {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            rx: Arc::new(Mutex::new(rx)),
            tx,
            count: Arc::new(Mutex::new(QueueSize::Limited(0))),
            capacity: Arc::new(Mutex::new(QueueSize::Limitless)),
        }
    }

    pub fn with_num_elements(num_elements: usize) -> Self {
        let (tx, rx) = channel();
        Self {
            rx: Arc::new(Mutex::new(rx)),
            tx,
            count: Arc::new(Mutex::new(QueueSize::Limited(0))),
            capacity: Arc::new(Mutex::new(QueueSize::Limited(num_elements))),
        }
    }
}
