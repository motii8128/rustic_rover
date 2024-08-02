use std::cell::RefCell;
use tokio::sync::mpsc;

pub struct ThreadConnector<T>
{
    pub publisher:RefCell<Option<mpsc::UnboundedSender<T>>>,
    pub subscriber:RefCell<Option<mpsc::UnboundedReceiver<T>>>
}

impl<T> ThreadConnector<T> {
    pub fn new()->ThreadConnector<T>
    {
        let (t,r) = mpsc::unbounded_channel::<T>();

        ThreadConnector { publisher: RefCell::new(Some(t)), subscriber: RefCell::new(Some(r)) }
    }
}