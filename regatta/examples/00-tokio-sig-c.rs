use futures::stream::{Stream, StreamExt};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::signal::unix::{signal, Signal, SignalKind};

#[derive(Debug)]
pub struct SignalStream {
    signal: Signal,
}

impl SignalStream {
    pub fn new(kind: SignalKind) -> Self {
        Self {
            signal: signal(kind).unwrap(),
        }
    }
}

impl Stream for SignalStream {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        self.signal.poll_recv(cx)
    }
}

pub mod mymod {

    use futures::StreamExt;

    pub async fn handle_signals(
        mut interrupts: futures::stream::Fuse<super::SignalStream>,
        mut terminates: futures::stream::Fuse<super::SignalStream>,
    ) {
        loop {
            futures::select! {
                s = interrupts.next() => {
                    println!("Trapped signal to quit. {:?}. Exiting!", s);
                    std::process::exit(1)
                },
                s = terminates.next() => {
                    println!("Trapped signal to quit. {:?}. Exiting!", s);
                    std::process::exit(1)
                },
                complete => break,
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let interrupts = SignalStream::new(SignalKind::interrupt()).fuse();
    let terminates = SignalStream::new(SignalKind::terminate()).fuse();

    let signals_task = tokio::spawn(mymod::handle_signals(interrupts, terminates));

    signals_task.await;
}
