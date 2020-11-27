use futures::{channel::mpsc, Future};
use std::pin::Pin;
use std::task::Poll;

pub(crate) type ExecutorFuture =
    Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + 'static>>;

pub struct Executor<I, T> {
    pub handle: fn(I) -> ExecutorFuture,
    pub future: Option<ExecutorFuture>,
    pub tx: Option<mpsc::Sender<T>>,
}

impl<I, T> Executor<I, T> {
    pub(crate) fn start(&mut self, get_input: Box<dyn Fn(mpsc::Receiver<T>) -> I>) -> bool {
        if self.future.is_none() {
            let (tx, rx) = mpsc::channel(1);
            self.tx = Some(tx);
            let input = get_input(rx);
            self.future = Some(Box::pin((self.handle)(input)));
            true
        } else {
            false
        }
    }

    pub(crate) fn send(&mut self, data: Option<T>) -> bool {
        if let Some(data) = data {
            self.tx.as_mut().unwrap().try_send(data).unwrap();
        } else {
            self.tx.take();
        }
        let mut context = std::task::Context::from_waker(futures::task::noop_waker_ref());
        match self.future.as_mut().unwrap().as_mut().poll(&mut context) {
            Poll::Pending => true,
            Poll::Ready(v) => v.is_ok(),
        }
    }
}
