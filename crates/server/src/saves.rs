#[cfg(test)]
use legatum_core::GameState;
#[cfg(test)]
use legatum_wasm::json;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

pub(crate) struct Saves {
    dir: PathBuf,
}

impl Saves {
    pub(crate) fn open(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    #[cfg(test)]
    pub(crate) fn load(&self) -> Result<Option<GameState>, String> {
        self.load_session().map(|s| s.map(|s| s.state))
    }

    pub(crate) fn load_session(&self) -> Result<Option<legatum_wasm::session::Session>, String> {
        let path = self.dir.join("autosave.json");
        match fs::read_to_string(&path) {
            Ok(save) => (|| {
                crate::host::check_state(&legatum_wasm::json::parse_state(&save)?)?;
                legatum_wasm::session::Session::from_save_json(&save, |_| {
                    legatum_wasm::session::BrainKind::Scripted
                })
            })()
            .map(Some)
            .map_err(|e| format!("cannot load {}: {e}", path.display())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("cannot read {}: {e}", path.display())),
        }
    }

    #[cfg(test)]
    pub(crate) fn write(&self, state: &GameState) -> io::Result<()> {
        self.write_with_prune(state, || self.prune())
    }

    #[cfg(test)]
    fn write_with_prune(
        &self,
        state: &GameState,
        prune: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        let save = json::state_to_json(state);
        self.write_json(state.turn, &save, prune)
    }

    pub(crate) fn write_session(&self, session: &legatum_wasm::session::Session) -> io::Result<()> {
        self.write_json(session.state.turn, &session.save_json(), || self.prune())
    }

    fn write_json(
        &self,
        turn: u32,
        save: &str,
        prune: impl FnOnce() -> io::Result<()>,
    ) -> io::Result<()> {
        atomic_write(&self.dir.join(format!("turn-{turn}.json")), save)?;
        prune()?;
        // This rename is the commit point. Nothing fallible may follow it: a rejected
        // new/load must leave the restart state matching the still-live game.
        atomic_write(&self.dir.join("autosave.json"), save)
    }

    /// A finished scenario run sent by the web or phone build, as `<saves>/scenarios/<name>`.
    /// The name is checked by the caller (`scenario_run_name`).
    pub(crate) fn write_scenario_run(&self, name: &str, json: &str) -> io::Result<PathBuf> {
        let dir = self.dir.join("scenarios");
        fs::create_dir_all(&dir)?;
        let path = dir.join(name);
        atomic_write(&path, json)?;
        Ok(path)
    }

    fn prune(&self) -> io::Result<()> {
        // Modification order also handles a new game or loading an earlier turn into this
        // directory: old high-numbered snapshots must not evict the new game's saves.
        let mut turns = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(turn) = name
                .strip_prefix("turn-")
                .and_then(|n| n.strip_suffix(".json"))
                .and_then(|n| n.parse::<u32>().ok())
                && entry.file_type()?.is_file()
            {
                turns.push((entry.metadata()?.modified()?, turn, entry.path()));
            }
        }
        turns.sort();
        let remove = turns.len().saturating_sub(10);
        for (_, _, path) in turns.into_iter().take(remove) {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

fn atomic_write(path: &Path, contents: &str) -> io::Result<()> {
    let temp = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temp)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::rename(temp, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use legatum_core::{Faction, GameSetup, new_game};

    #[test]
    fn keeps_ten_recent_snapshots_even_when_loading_an_earlier_turn() {
        let root = tempfile::tempdir().unwrap();
        let saves = Saves::open(root.path().to_owned()).unwrap();
        let mut state = new_game(&GameSetup::standard(
            u64::MAX,
            [Faction::India, Faction::Byzantium],
        ));
        fs::write(root.path().join("manual.json"), "leave this alone").unwrap();
        for turn in 2..=13 {
            state.turn = turn;
            saves.write(&state).unwrap();
        }
        for turn in 2..=13 {
            assert_eq!(
                root.path().join(format!("turn-{turn}.json")).exists(),
                turn >= 4
            );
        }
        assert_eq!(saves.load().unwrap().unwrap().hash(), state.hash());
        state.turn = 1;
        saves.write(&state).unwrap();
        assert!(root.path().join("turn-1.json").exists());
        assert!(!root.path().join("turn-4.json").exists());
        assert_eq!(saves.load().unwrap().unwrap().hash(), state.hash());
        assert_eq!(
            fs::read_to_string(root.path().join("manual.json")).unwrap(),
            "leave this alone"
        );
        let snapshots = fs::read_dir(root.path())
            .unwrap()
            .filter(|entry| {
                entry
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("turn-")
            })
            .count();
        assert_eq!(snapshots, 10);
    }

    #[test]
    fn corrupt_autosave_is_reported_without_overwriting_it() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("autosave.json");
        fs::write(&path, "corrupt").unwrap();
        let saves = Saves::open(root.path().to_owned()).unwrap();
        assert!(saves.load().unwrap_err().contains("autosave.json"));
        assert_eq!(fs::read_to_string(path).unwrap(), "corrupt");
    }

    #[test]
    fn snapshot_failure_preserves_the_committed_autosave() {
        let root = tempfile::tempdir().unwrap();
        let saves = Saves::open(root.path().to_owned()).unwrap();
        let mut state = new_game(&GameSetup::standard(
            7,
            [Faction::India, Faction::Byzantium],
        ));
        saves.write(&state).unwrap();
        let committed = fs::read(root.path().join("autosave.json")).unwrap();
        state.turn += 1;
        fs::create_dir(root.path().join(format!("turn-{}.json", state.turn))).unwrap();
        assert!(saves.write(&state).is_err());
        assert_eq!(
            fs::read(root.path().join("autosave.json")).unwrap(),
            committed
        );
        assert_eq!(saves.load().unwrap().unwrap().turn, 1);
    }

    #[test]
    fn snapshot_pruning_failure_preserves_the_committed_autosave() {
        let root = tempfile::tempdir().unwrap();
        let saves = Saves::open(root.path().to_owned()).unwrap();
        let mut state = new_game(&GameSetup::standard(
            7,
            [Faction::India, Faction::Byzantium],
        ));
        saves.write(&state).unwrap();
        let committed = fs::read(root.path().join("autosave.json")).unwrap();
        state.turn += 1;
        // Inject the retention failure so the test is independent of platform and uid.
        let result = saves.write_with_prune(&state, || Err(io::Error::other("cannot prune")));
        assert!(result.is_err());
        assert!(root.path().join("turn-2.json").is_file());
        assert_eq!(
            fs::read(root.path().join("autosave.json")).unwrap(),
            committed
        );
    }
}
