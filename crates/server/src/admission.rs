//! Admission runs before upgrade or blocking game work. Permits cover closing sockets too.
use std::{
    collections::BTreeMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub(crate) const PRE_JOIN_TIMEOUT: Duration = Duration::from_secs(30);
const PER_PEER: usize = 8;
const RESERVED_LOCAL: usize = 4;
const WATCHERS: usize = 16;

pub(crate) struct Admission {
    all: Arc<Semaphore>,
    remote: Arc<Semaphore>,
    watchers: Arc<Semaphore>,
    peers: Mutex<BTreeMap<IpAddr, usize>>,
}

pub(crate) struct Permit {
    admission: Arc<Admission>,
    ip: IpAddr,
    local: bool,
    _all: OwnedSemaphorePermit,
    _remote: Option<OwnedSemaphorePermit>,
    watcher: Option<OwnedSemaphorePermit>,
}

impl Admission {
    pub(crate) fn new() -> Self {
        Self {
            all: Arc::new(Semaphore::new(crate::outbox::MAX_CONNECTIONS)),
            remote: Arc::new(Semaphore::new(
                crate::outbox::MAX_CONNECTIONS - RESERVED_LOCAL,
            )),
            watchers: Arc::new(Semaphore::new(WATCHERS)),
            peers: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn acquire(self: &Arc<Self>, ip: IpAddr, local: bool) -> Option<Permit> {
        let ip = ip.to_canonical();
        let mut peers = self.peers.lock().ok()?;
        // Loopback may represent all browsers on the host, including local test players.
        let limit = if local {
            crate::outbox::MAX_CONNECTIONS
        } else {
            PER_PEER
        };
        if peers.get(&ip).copied().unwrap_or(0) >= limit {
            return None;
        }
        let all = self.all.clone().try_acquire_owned().ok()?;
        let remote = if local {
            None
        } else {
            Some(self.remote.clone().try_acquire_owned().ok()?)
        };
        let watcher = if local {
            None
        } else {
            Some(self.watchers.clone().try_acquire_owned().ok()?)
        };
        *peers.entry(ip).or_default() += 1;
        Some(Permit {
            admission: self.clone(),
            ip,
            local,
            _all: all,
            _remote: remote,
            watcher,
        })
    }
}

impl Permit {
    pub(crate) fn admitted(&mut self, admitted: bool) -> bool {
        if admitted {
            self.watcher = None;
        } else if !self.local && self.watcher.is_none() {
            let Ok(permit) = self.admission.watchers.clone().try_acquire_owned() else {
                return false;
            };
            self.watcher = Some(permit);
        }
        true
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        let mut peers = self.admission.peers.lock().expect("admission lock");
        let count = peers.get_mut(&self.ip).expect("admitted peer");
        *count -= 1;
        if *count == 0 {
            peers.remove(&self.ip);
        }
    }
}
