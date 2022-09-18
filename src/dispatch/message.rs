use std::fmt::Debug;

pub trait Message: Debug + Clone + Send + Sync + 'static {}
impl<T: Debug + Clone + Send + Sync + 'static> Message for T {}
