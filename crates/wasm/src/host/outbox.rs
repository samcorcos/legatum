//! Each connection's outgoing messages. The game logic fills them; the transport (the browser
//! host's JavaScript, or the tests) drains them after every operation. Nothing here waits.
use super::protocol::ServerMessage;
use std::{cell::RefCell, collections::VecDeque, rc::Rc, sync::Arc};

pub(crate) const MAX_CONNECTIONS: usize = 32;
pub(crate) const QUEUE_CAPACITY: usize = 16;
pub(crate) const QUEUE_BYTES: usize = 8 * 1024 * 1024;
/// Map pings waiting for one connection; a new one pushes the oldest out.
pub(crate) const PING_CAPACITY: usize = 8;

#[derive(Clone, Debug)]
pub(crate) struct Close {
    pub(crate) code: Option<&'static str>,
    pub(crate) message: String,
}

/// One replaceable snapshot: only the latest value is written, and an identical one is not.
#[derive(Default)]
struct Latest {
    value: Option<Arc<str>>,
    changed: bool,
}

impl Latest {
    fn set(&mut self, value: Option<&Arc<str>>) {
        if self.value.as_ref() != value {
            self.value = value.cloned();
            self.changed = true;
        }
    }

    fn take(&mut self) -> Option<Arc<str>> {
        std::mem::take(&mut self.changed)
            .then(|| self.value.clone())
            .flatten()
    }
}

#[derive(Default)]
struct Inner {
    // A turn and its view share a delivery so seat snapshots cannot split the pair.
    reliable: VecDeque<Vec<String>>,
    bytes: usize,
    seats: Latest,
    staging: Latest,
    /// Teammates' map pings, newest last: only the latest few are kept.
    pings: VecDeque<Arc<str>>,
    close: Option<Close>,
    admitted: bool,
    admitted_changed: bool,
}

/// The game's end of a connection. Dropping it without `close` ends the connection silently.
pub(crate) struct Sender(Rc<RefCell<Inner>>);

/// The transport's end of a connection.
pub(crate) struct Receiver(Rc<RefCell<Inner>>);

pub(crate) fn channel() -> (Sender, Receiver) {
    let inner = Rc::new(RefCell::new(Inner::default()));
    (Sender(inner.clone()), Receiver(inner))
}

impl Sender {
    pub(crate) fn admitted(&self, value: bool) {
        let mut inner = self.0.borrow_mut();
        if inner.admitted != value {
            inner.admitted = value;
            inner.admitted_changed = true;
        }
    }

    /// Queues one delivery. Fails when the connection already holds 16 deliveries or 8 MiB.
    pub(crate) fn send(&self, messages: &[ServerMessage]) -> Result<(), ()> {
        let texts: Vec<_> = messages
            .iter()
            .map(|message| serde_json::to_string(message).expect("protocol serializes"))
            .collect();
        let bytes: usize = texts.iter().map(String::len).sum();
        let mut inner = self.0.borrow_mut();
        if inner.reliable.len() >= QUEUE_CAPACITY || inner.bytes + bytes > QUEUE_BYTES {
            return Err(());
        }
        inner.bytes += bytes;
        inner.reliable.push_back(texts);
        Ok(())
    }

    pub(crate) fn seats(&self, snapshot: &Arc<str>) {
        self.0.borrow_mut().seats.set(Some(snapshot));
    }

    pub(crate) fn ping(&self, text: &Arc<str>) {
        let mut inner = self.0.borrow_mut();
        if inner.pings.len() >= PING_CAPACITY {
            inner.pings.pop_front();
        }
        inner.pings.push_back(text.clone());
    }

    pub(crate) fn staging(&self, snapshot: Option<&Arc<str>>) {
        self.0.borrow_mut().staging.set(snapshot);
    }

    /// Ends the connection: queued deliveries are dropped and only the reason is written.
    pub(crate) fn close(self, code: Option<&'static str>, message: &str) {
        self.0.borrow_mut().close = Some(Close {
            code,
            message: message.into(),
        });
    }
}

/// What a drain hands the transport for one connection.
pub(crate) struct Drained {
    pub(crate) texts: Vec<String>,
    /// The connection is over: write `texts`, then close it.
    pub(crate) closed: bool,
}

impl Receiver {
    /// A change in whether the connection holds a seat, if there was one since the last call.
    pub(crate) fn admitted(&self) -> Option<bool> {
        let mut inner = self.0.borrow_mut();
        std::mem::take(&mut inner.admitted_changed).then_some(inner.admitted)
    }

    /// Everything ready to write, in the order the socket wrote it before: reliable deliveries
    /// first, then the latest seat and staging snapshots. A closed connection writes only its
    /// terminal error, and one whose sender is gone writes nothing more.
    pub(crate) fn drain(&self) -> Drained {
        let mut inner = self.0.borrow_mut();
        if let Some(Close { code, message }) = inner.close.take() {
            let text = serde_json::to_string(&ServerMessage::Error { code, message })
                .expect("protocol serializes");
            return Drained {
                texts: vec![text],
                closed: true,
            };
        }
        let mut texts: Vec<String> = inner.reliable.drain(..).flatten().collect();
        inner.bytes = 0;
        texts.extend(inner.pings.drain(..).map(|p| p.to_string()));
        texts.extend(inner.seats.take().map(|s| s.to_string()));
        texts.extend(inner.staging.take().map(|s| s.to_string()));
        drop(inner);
        Drained {
            texts,
            closed: Rc::strong_count(&self.0) == 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_close_replaces_queued_deliveries_with_its_reason() {
        let (sender, receiver) = channel();
        sender
            .send(&[ServerMessage::Pong])
            .expect("room in the queue");
        sender.close(Some("seat_taken"), "replaced");
        let drained = receiver.drain();
        assert!(drained.closed);
        assert_eq!(drained.texts.len(), 1);
        assert!(drained.texts[0].contains("seat_taken"));
    }

    #[test]
    fn the_queue_has_a_delivery_and_byte_budget_that_a_drain_returns() {
        let (sender, receiver) = channel();
        let message = ServerMessage::Error {
            code: None,
            message: "x".repeat(QUEUE_BYTES / 2),
        };
        sender.send(std::slice::from_ref(&message)).unwrap();
        assert!(sender.send(std::slice::from_ref(&message)).is_err());
        assert_eq!(receiver.drain().texts.len(), 1);
        sender.send(std::slice::from_ref(&message)).unwrap();
        receiver.drain();
        for _ in 0..QUEUE_CAPACITY {
            sender.send(&[ServerMessage::Pong]).unwrap();
        }
        assert!(sender.send(&[ServerMessage::Pong]).is_err());
    }

    #[test]
    fn snapshots_coalesce_follow_deliveries_and_identical_updates_are_not_rewritten() {
        let (sender, receiver) = channel();
        sender.seats(&Arc::from("first"));
        sender.seats(&Arc::from("latest"));
        sender.send(&[ServerMessage::Pong]).unwrap();
        let drained = receiver.drain();
        assert_eq!(drained.texts, [r#"{"type":"pong"}"#, "latest"]);
        assert!(!drained.closed);
        sender.seats(&Arc::from("latest"));
        assert!(receiver.drain().texts.is_empty());
        // A cleared staging snapshot writes nothing, then a new one is written again.
        sender.staging(Some(&Arc::from("room")));
        sender.staging(None);
        assert!(receiver.drain().texts.is_empty());
        sender.staging(Some(&Arc::from("room")));
        assert_eq!(receiver.drain().texts, ["room"]);
    }

    #[test]
    fn map_pings_keep_only_the_latest_few_and_never_fill_the_reliable_queue() {
        let (sender, receiver) = channel();
        for i in 0..PING_CAPACITY + 3 {
            sender.ping(&Arc::from(format!("p{i}")));
        }
        for _ in 0..QUEUE_CAPACITY {
            sender.send(&[ServerMessage::Pong]).unwrap();
        }
        let texts = receiver.drain().texts;
        let pings: Vec<_> = texts
            .iter()
            .filter(|t| t.starts_with('p') && *t != r#"{"type":"pong"}"#)
            .collect();
        assert_eq!(pings.len(), PING_CAPACITY);
        assert_eq!(pings[0], "p3");
    }

    #[test]
    fn a_dropped_sender_ends_the_connection_silently() {
        let (sender, receiver) = channel();
        drop(sender);
        let drained = receiver.drain();
        assert!(drained.closed && drained.texts.is_empty());
    }
}
