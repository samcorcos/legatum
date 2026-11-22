//! Bounded, nonblocking handoff from the game mutex to each socket task.
use crate::protocol::ServerMessage;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc, watch};

pub(crate) const MAX_CONNECTIONS: usize = 32;
pub(crate) const QUEUE_CAPACITY: usize = 16;
pub(crate) const QUEUE_BYTES: usize = 8 * 1024 * 1024;

pub(crate) struct Outbound {
    // A turn and its view share a delivery so seat snapshots cannot split the pair.
    pub(crate) texts: Vec<String>,
    // Includes the in-flight write; released only after delivery or cancellation.
    _bytes: OwnedSemaphorePermit,
}

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) code: Option<&'static str>,
    pub(crate) message: String,
}

pub(crate) struct Sender {
    reliable: mpsc::Sender<Outbound>,
    budget: Arc<Semaphore>,
    seats: watch::Sender<Option<Arc<str>>>,
    staging: watch::Sender<Option<Arc<str>>>,
    close: watch::Sender<Option<Close>>,
    admitted: watch::Sender<bool>,
}

pub(crate) struct Receiver {
    pub(crate) reliable: mpsc::Receiver<Outbound>,
    pub(crate) seats: watch::Receiver<Option<Arc<str>>>,
    pub(crate) staging: watch::Receiver<Option<Arc<str>>>,
    pub(crate) close: watch::Receiver<Option<Close>>,
    pub(crate) admitted: watch::Receiver<bool>,
}

pub(crate) async fn until_closed(
    close: &mut watch::Receiver<Option<Close>>,
    work: impl std::future::Future<Output = ()>,
) -> Option<Close> {
    tokio::select! {
        biased;
        _ = close.changed() => {},
        () = work => {},
    }
    // A close can be published after its branch polls Pending but before work
    // observes another closed channel and finishes. Biased polling alone cannot
    // prevent this; always read the terminal reason after either branch completes.
    close.borrow().clone()
}

pub(crate) fn channel() -> (Sender, Receiver) {
    let (reliable, receiver) = mpsc::channel(QUEUE_CAPACITY);
    let (seats, seat_receiver) = watch::channel(None);
    let (staging, staging_receiver) = watch::channel(None);
    let (close, close_receiver) = watch::channel(None);
    let (admitted, admitted_receiver) = watch::channel(false);
    (
        Sender {
            reliable,
            budget: Arc::new(Semaphore::new(QUEUE_BYTES)),
            seats,
            staging,
            close,
            admitted,
        },
        Receiver {
            reliable: receiver,
            seats: seat_receiver,
            staging: staging_receiver,
            close: close_receiver,
            admitted: admitted_receiver,
        },
    )
}

impl Sender {
    pub(crate) fn admitted(&self, value: bool) {
        self.admitted.send_if_modified(|current| {
            if *current == value {
                return false;
            }
            *current = value;
            true
        });
    }
    pub(crate) fn send(&self, messages: &[ServerMessage]) -> Result<(), ()> {
        let texts: Vec<_> = messages
            .iter()
            .map(|message| serde_json::to_string(message).expect("protocol serializes"))
            .collect();
        let bytes: usize = texts.iter().map(String::len).sum();
        let bytes = u32::try_from(bytes).map_err(|_| ())?;
        let permit = self
            .budget
            .clone()
            .try_acquire_many_owned(bytes)
            .map_err(|_| ())?;
        self.reliable
            .try_send(Outbound {
                texts,
                _bytes: permit,
            })
            .map_err(|_| ())
    }

    pub(crate) fn seats(&self, snapshot: &Arc<str>) {
        self.seats.send_if_modified(|current| {
            if current.as_ref() == Some(snapshot) {
                return false;
            }
            *current = Some(snapshot.clone());
            true
        });
    }

    pub(crate) fn close(self, code: Option<&'static str>, message: &str) {
        // Independent of queue capacity; wakes and cancels a stalled socket write.
        self.close.send_replace(Some(Close {
            code,
            message: message.into(),
        }));
    }

    pub(crate) fn staging(&self, snapshot: Option<&Arc<str>>) {
        self.staging.send_if_modified(|current| {
            if current.as_ref() == snapshot {
                return false;
            }
            *current = snapshot.cloned();
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn terminal_reason_survives_sender_drop_during_the_work_poll() {
        let (sender, mut receiver) = channel();
        let reason = until_closed(&mut receiver.close, async {
            // The close branch has already polled Pending. A takeover now publishes
            // the reason and drops all senders while the connection work is polled.
            sender.close(Some("seat_taken"), "replaced");
            assert!(receiver.admitted.changed().await.is_err());
        })
        .await
        .expect("work completion discarded the terminal close reason");
        assert_eq!(reason.code, Some("seat_taken"));
        assert_eq!(reason.message, "replaced");
    }

    #[test]
    fn byte_budget_covers_in_flight_deliveries_and_returns_on_drop() {
        let (sender, mut receiver) = channel();
        let message = ServerMessage::Error {
            code: None,
            message: "x".repeat(QUEUE_BYTES / 2),
        };
        sender.send(std::slice::from_ref(&message)).unwrap();
        let in_flight = receiver.reliable.try_recv().unwrap();
        assert!(sender.send(std::slice::from_ref(&message)).is_err());
        drop(in_flight);
        sender.send(std::slice::from_ref(&message)).unwrap();
        sender.close(Some("slow_consumer"), "queue full");
        assert_eq!(
            receiver.close.borrow().as_ref().unwrap().code,
            Some("slow_consumer")
        );
    }

    #[test]
    fn seat_snapshots_coalesce_and_identical_updates_do_not_wake_the_writer() {
        let (sender, mut receiver) = channel();
        sender.seats(&Arc::from("first"));
        sender.seats(&Arc::from("latest"));
        assert_eq!(
            receiver.seats.borrow_and_update().as_deref(),
            Some("latest")
        );
        sender.seats(&Arc::from("latest"));
        assert!(!receiver.seats.has_changed().unwrap());
        assert!(receiver.reliable.try_recv().is_err());
    }
}
