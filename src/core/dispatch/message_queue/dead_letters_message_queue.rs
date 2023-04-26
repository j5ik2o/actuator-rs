use crate::core::actor::actor_ref::{ActorRef, ActorRefBehavior};
use crate::core::dispatch::any_message::AnyMessage;
use crate::core::dispatch::envelope::Envelope;
use crate::core::dispatch::message_queue::{
  MessageQueueBehavior, MessageQueueRWFactoryBehavior, MessageQueueReaderBehavior, MessageQueueReaderFactoryBehavior,
  MessageQueueSize, MessageQueueWithRWFactoryBehavior, MessageQueueWriterBehavior, MessageQueueWriterFactoryBehavior,
};

#[derive(Debug, Clone)]
pub struct DeadLettersMessageQueue {
  dead_letter_ref: ActorRef<AnyMessage>,
}

impl DeadLettersMessageQueue {
  pub fn new(dead_letter_ref: ActorRef<AnyMessage>) -> Self {
    DeadLettersMessageQueue { dead_letter_ref }
  }
}

unsafe impl Send for DeadLettersMessageQueue {}
unsafe impl Sync for DeadLettersMessageQueue {}

#[derive(Debug, Clone)]
pub struct DeadLettersMessageQueueWriter {
  underlying: DeadLettersMessageQueue,
}

unsafe impl Send for DeadLettersMessageQueueWriter {}
unsafe impl Sync for DeadLettersMessageQueueWriter {}

#[derive(Debug, Clone)]
pub struct DeadLettersMessageQueueReader {
  underlying: DeadLettersMessageQueue,
}

unsafe impl Send for DeadLettersMessageQueueReader {}
unsafe impl Sync for DeadLettersMessageQueueReader {}

impl MessageQueueBehavior<AnyMessage> for DeadLettersMessageQueue {
  fn number_of_messages(&self) -> MessageQueueSize {
    MessageQueueSize::Limitless
  }

  fn has_messages(&self) -> bool {
    true
  }
}

impl MessageQueueRWFactoryBehavior<AnyMessage> for DeadLettersMessageQueue {}

impl MessageQueueWriterFactoryBehavior<AnyMessage> for DeadLettersMessageQueue {
  type Writer = DeadLettersMessageQueueWriter;

  fn writer(&self) -> Self::Writer {
    DeadLettersMessageQueueWriter {
      underlying: self.clone(),
    }
  }
}

impl MessageQueueReaderFactoryBehavior<AnyMessage> for DeadLettersMessageQueue {
  type Reader = DeadLettersMessageQueueReader;

  fn reader(&self) -> Self::Reader {
    DeadLettersMessageQueueReader {
      underlying: self.clone(),
    }
  }
}

impl MessageQueueWithRWFactoryBehavior<AnyMessage> for DeadLettersMessageQueue {}

// ---

impl MessageQueueBehavior<AnyMessage> for DeadLettersMessageQueueWriter {
  fn number_of_messages(&self) -> MessageQueueSize {
    self.underlying.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    self.underlying.has_messages()
  }
}

impl MessageQueueWriterBehavior<AnyMessage> for DeadLettersMessageQueueWriter {
  fn enqueue(&mut self, _receiver: ActorRef<AnyMessage>, handle: Envelope) -> anyhow::Result<()> {
    self.underlying.dead_letter_ref.tell(handle.message);
    Ok(())
  }
}

// ---

impl MessageQueueBehavior<AnyMessage> for DeadLettersMessageQueueReader {
  fn number_of_messages(&self) -> MessageQueueSize {
    self.underlying.number_of_messages()
  }

  fn has_messages(&self) -> bool {
    self.underlying.has_messages()
  }
}

impl MessageQueueReaderBehavior<AnyMessage> for DeadLettersMessageQueueReader {
  fn dequeue(&mut self) -> anyhow::Result<Option<Envelope>> {
    Err(anyhow::anyhow!(
      "DeadLettersMessageQueueReader::dequeue() is not supported."
    ))
  }
}
