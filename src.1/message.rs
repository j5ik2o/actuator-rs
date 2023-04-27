use std::fmt::Debug;

pub trait Message: Debug + Clone + PartialEq + Send + Sync + 'static {}

impl<T: Debug + Clone + PartialEq + Send + Sync + 'static> Message for T {}
