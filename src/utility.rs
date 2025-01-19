use std::sync::{Arc, Mutex};
pub type AM<T> = Arc<Mutex<T>>;
pub type AMV<T> = Arc<Mutex<Vec<T>>>;
pub fn sync<T>(obj: T) -> AM<T> { Arc::new(Mutex::new(obj)) }
