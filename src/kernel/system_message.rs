#[derive(Clone, Debug)]
pub enum SystemMessage {
  Command(ActorCommand),
  Event(ActorEvent),
  Failed,
}

#[derive(Clone, Debug)]
pub enum ActorCommand {
  CreateActor,
  TerminateActor,
  RestartActor,
}

#[derive(Clone, Debug)]
pub enum ActorEvent {
  ActorCreated,
  ActorTerminated,
  ActorRestarted,
}
