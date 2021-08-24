pub enum SystemMessage {
  Command(ActorCommand),
  Event(ActorEvent),
  Failed,
}

pub enum ActorCommand {
  CreateActor,
  TerminateActor,
  RestartActor,
}

pub enum ActorEvent {
  ActorCreated,
  ActorTerminated,
  ActorRestarted,
}
