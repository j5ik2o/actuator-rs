use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::*;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct QueueVec<E> {
    values: Arc<Mutex<VecDeque<E>>>,
    pub(crate) capacity: QueueSize,
}

impl<E: Element> QueueVec<E> {
    pub fn new() -> Self {
        Self {
            values: Arc::new(Mutex::new(VecDeque::new())),
            capacity: QueueSize::Limitless,
        }
    }

    pub fn with_num_elements(num_elements: usize) -> Self {
        Self {
            values: Arc::new(Mutex::new(VecDeque::new())),
            capacity: QueueSize::Limited(num_elements),
        }
    }

    pub fn with_elements(values: impl IntoIterator<Item = E> + ExactSizeIterator) -> Self {
        let num_elements = values.len();
        let vec = values.into_iter().collect::<VecDeque<E>>();
        Self {
            values: Arc::new(Mutex::new(vec)),
            capacity: QueueSize::Limited(num_elements),
        }
    }
}

impl<E: Element + 'static> QueueBehavior<E> for QueueVec<E> {
    fn len(&self) -> QueueSize {
        let mg = self.values.lock().unwrap();
        let len = mg.len();
        QueueSize::Limited(len)
    }

    fn capacity(&self) -> QueueSize {
        self.capacity.clone()
    }

    fn offer(&mut self, e: E) -> Result<()> {
        if self.non_full() {
            let mut mg = self.values.lock().unwrap();
            mg.push_back(e);
            Ok(())
        } else {
            Err(anyhow::Error::new(QueueError::OfferError(e)))
        }
    }

    fn poll(&mut self) -> Result<Option<E>> {
        let mut mg = self.values.lock().unwrap();
        Ok(mg.pop_front())
    }
}

impl<E: Element + 'static> HasPeekBehavior<E> for QueueVec<E> {
    fn peek(&self) -> Result<Option<E>> {
        let mg = self.values.lock().unwrap();
        Ok(mg.front().map(|e| e.clone()))
    }
}
