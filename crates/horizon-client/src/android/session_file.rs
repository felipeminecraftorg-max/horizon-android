//! A prepared read-once handoff belongs to the launcher until the game takes it.
//! Dropping a worker result, switching account/profile or closing the launcher
//! must remove an unconsumed file, including a result already queued in a channel.
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct SessionFile(Option<PathBuf>);

impl From<PathBuf> for SessionFile {
    fn from(path: PathBuf) -> Self {
        Self(Some(path))
    }
}

impl SessionFile {
    pub fn into_path(mut self) -> PathBuf {
        self.0.take().expect("the handoff is taken only once")
    }
}

impl Drop for SessionFile {
    fn drop(&mut self) {
        if let Some(path) = self.0.take()
            && let Err(error) = std::fs::remove_file(path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            // Never print session contents, credentials or the player's path.
            log::warn!(
                "Could not remove an unused account handoff: {}",
                error.kind()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn file() -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "horizon-unused-session-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, "test handoff, not a credential").unwrap();
        path
    }

    #[test]
    fn switching_or_closing_removes_unused_handoff() {
        let old = file();
        let new = file();
        let mut slot = Some(SessionFile::from(old.clone()));
        assert!(slot.is_some());
        slot = Some(SessionFile::from(new.clone()));
        assert!(!old.exists());
        assert!(new.exists());
        drop(slot);
        assert!(!new.exists());
    }

    #[test]
    fn cancelled_worker_also_cleans_an_already_queued_result() {
        let path = file();
        let (sender, receiver) = std::sync::mpsc::channel();
        sender.send(SessionFile::from(path.clone())).unwrap();
        drop(receiver);
        assert!(!path.exists());
        let path = file();
        drop(sender.send(SessionFile::from(path.clone())));
        assert!(!path.exists());
    }

    #[test]
    fn launch_transfers_the_file_to_the_existing_read_once_consumer() {
        let path = file();
        let handed_off = SessionFile::from(path.clone()).into_path();
        assert!(path.exists());
        assert_eq!(handed_off, path);
        std::fs::remove_file(handed_off).unwrap();
    }
}
