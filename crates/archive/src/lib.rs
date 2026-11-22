//! Local, compressed game archive. No network data or credentials enter this API.
use fs2::FileExt;
pub use legatum_wasm::history::{Metadata, Recording, Source};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

const MAX_JSON_BYTES: u64 = 64 * 1024 * 1024;

/// The per-user data folder for files the app and server write when no flag says otherwise:
/// `~/Library/Application Support/Legatum` on macOS, `%APPDATA%\Legatum` on Windows, and
/// `$XDG_DATA_HOME/legatum` (default `~/.local/share/legatum`) elsewhere. `None` when the
/// environment names no home folder at all.
pub fn data_dir() -> Option<PathBuf> {
    data_dir_from(std::env::consts::OS, |key| std::env::var_os(key))
}

fn data_dir_from(os: &str, var: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    let set = |key: &str| var(key).filter(|v| !v.is_empty()).map(PathBuf::from);
    match os {
        "macos" | "ios" => set("HOME").map(|h| h.join("Library/Application Support/Legatum")),
        "windows" => set("APPDATA")
            .or_else(|| set("USERPROFILE").map(|p| p.join("AppData").join("Roaming")))
            .map(|p| p.join("Legatum")),
        _ => set("XDG_DATA_HOME")
            .filter(|p| p.is_absolute())
            .or_else(|| set("HOME").map(|h| h.join(".local/share")))
            .map(|p| p.join("legatum")),
    }
}

/// Resolve the flag, then LEGATUM_ARCHIVE, then `~/LegatumData/archive`, then
/// [`data_dir`]`/archive` (Windows, where HOME is normally unset).
pub fn archive_root(flag: Option<PathBuf>) -> Result<PathBuf, String> {
    let path = flag
        .or_else(|| std::env::var_os("LEGATUM_ARCHIVE").map(PathBuf::from))
        .or_else(|| {
            std::env::var_os("HOME")
                .filter(|h| !h.is_empty())
                .map(|p| PathBuf::from(p).join("LegatumData/archive"))
        })
        .or_else(|| data_dir().map(|d| d.join("archive")))
        .ok_or_else(|| "no home folder found; pass --archive-dir".to_owned())?;
    outside_repo(&path)
}

/// Canonicalize existing ancestors, including symlinks, before creating any directories.
///
/// Refuses a path inside a Legatum source checkout, or inside any `.git` folder: archives are
/// large binary tapes, and one written into the working tree gets committed by accident. Any
/// other git repository is fine, so a home folder tracked with git still works.
pub fn outside_repo(path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(path)
    };
    let mut existing = absolute.as_path();
    let mut suffix = Vec::new();
    while !existing.exists() {
        suffix.push(
            existing
                .file_name()
                .ok_or("invalid archive path")?
                .to_owned(),
        );
        existing = existing.parent().ok_or("invalid archive parent")?;
    }
    let mut resolved = existing.canonicalize().map_err(|e| e.to_string())?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    for parent in resolved.ancestors() {
        if parent.file_name().is_some_and(|p| p == ".git") || is_legatum_checkout(parent) {
            return Err(format!(
                "archive must be outside the Legatum source checkout: {}",
                resolved.display()
            ));
        }
    }
    Ok(resolved)
}

/// A git working tree (or worktree, whose `.git` is a file) holding this crate's source.
fn is_legatum_checkout(dir: &Path) -> bool {
    dir.join(".git").exists() && dir.join("crates/archive/src/lib.rs").is_file()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<OsString> + 'a {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| OsString::from(v))
        }
    }

    #[test]
    fn data_dir_follows_each_platform() {
        let home = [("HOME", "/home/a")];
        assert_eq!(
            data_dir_from("macos", env(&home)),
            Some(PathBuf::from("/home/a/Library/Application Support/Legatum"))
        );
        assert_eq!(
            data_dir_from("linux", env(&home)),
            Some(PathBuf::from("/home/a/.local/share/legatum"))
        );
        assert_eq!(
            data_dir_from(
                "linux",
                env(&[("HOME", "/home/a"), ("XDG_DATA_HOME", "/data")])
            ),
            Some(PathBuf::from("/data/legatum"))
        );
        // A relative XDG_DATA_HOME is invalid per the spec and is ignored.
        assert_eq!(
            data_dir_from(
                "linux",
                env(&[("HOME", "/home/a"), ("XDG_DATA_HOME", "rel")])
            ),
            Some(PathBuf::from("/home/a/.local/share/legatum"))
        );
        // Windows has no HOME: APPDATA, else the profile's Roaming folder.
        assert_eq!(
            data_dir_from(
                "windows",
                env(&[("APPDATA", "C:/Profiles/a/AppData/Roaming")])
            ),
            Some(PathBuf::from("C:/Profiles/a/AppData/Roaming").join("Legatum"))
        );
        assert_eq!(
            data_dir_from("windows", env(&[("USERPROFILE", "C:/Profiles/a")])),
            Some(
                PathBuf::from("C:/Profiles/a")
                    .join("AppData")
                    .join("Roaming")
                    .join("Legatum")
            )
        );
        assert_eq!(data_dir_from("linux", env(&[("HOME", "")])), None);
        assert_eq!(data_dir_from("windows", env(&home)), None);
    }

    #[test]
    fn only_a_legatum_checkout_is_refused() {
        let root = tempfile::tempdir().unwrap();
        // A home folder tracked with git (dotfiles) is an ordinary place for the archive.
        let home = root.path().join("home");
        fs::create_dir_all(home.join(".git")).unwrap();
        assert!(outside_repo(&home.join("LegatumData/archive")).is_ok());
        // Inside a Legatum checkout, or a worktree of one (`.git` is a file), it is refused.
        let checkout = home.join("code/legatum");
        fs::create_dir_all(checkout.join("crates/archive/src")).unwrap();
        fs::write(checkout.join("crates/archive/src/lib.rs"), "").unwrap();
        fs::write(checkout.join(".git"), "gitdir: elsewhere").unwrap();
        assert!(outside_repo(&checkout.join("archive")).is_err());
        assert!(outside_repo(&home.join(".git/archive")).is_err());
    }
}

pub fn encode(recording: &Recording) -> Result<Vec<u8>, String> {
    recording.validate()?;
    let mut encoder =
        zstd::stream::write::Encoder::new(Vec::new(), 9).map_err(|e| e.to_string())?;
    encoder.include_checksum(true).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut encoder, recording).map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())
}

pub fn read(path: &Path) -> Result<Recording, String> {
    let file = File::open(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let reader: Box<dyn Read> = match path.extension().and_then(|s| s.to_str()) {
        Some("zst") => {
            Box::new(zstd::stream::read::Decoder::new(file).map_err(|e| format!("zstd: {e}"))?)
        }
        Some("gz") => Box::new(flate2::read::GzDecoder::new(file)),
        Some("json") => Box::new(file),
        _ => return Err("expected .tape.json.zst, .tape.json.gz or .json".into()),
    };
    let mut bytes = Vec::new();
    reader
        .take(MAX_JSON_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("corrupt archive: {e}"))?;
    if bytes.len() as u64 > MAX_JSON_BYTES {
        return Err("archive exceeds 64 MiB decompressed limit".into());
    }
    let recording: Recording =
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid archive JSON: {e}"))?;
    recording.validate()?;
    Ok(recording)
}

#[derive(Default)]
struct Index {
    offset: u64,
    ids: HashSet<String>,
}

#[derive(Clone)]
pub struct Archive {
    root: PathBuf,
    index: Arc<Mutex<Index>>,
}

impl Archive {
    pub fn open(root: PathBuf) -> Result<Self, String> {
        let root = outside_repo(&root)?;
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        Ok(Self {
            root,
            index: Arc::default(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn relative_path(recording: &Recording) -> Result<PathBuf, String> {
        recording.validate()?;
        let at = &recording.metadata.created_at;
        // RFC3339 was parsed in validate; restrict the path to ordinary four-digit years.
        if at.len() < 7 || !at.as_bytes()[..4].iter().all(u8::is_ascii_digit) {
            return Err("invalid archive year".into());
        }
        let mut path = PathBuf::new();
        if recording.metadata.finished_at.is_none() {
            path.push("in-progress");
        }
        path.push(&at[..4]);
        path.push(&at[5..7]);
        path.push(format!("{}.tape.json.zst", recording.metadata.game_id));
        Ok(path)
    }

    /// All processes share a filesystem lock, and each writer scans only new index lines.
    /// Retrying after file rename or index append is safe; final files are immutable.
    pub fn write(&self, recording: &Recording) -> Result<PathBuf, String> {
        let bytes = encode(recording)?;
        let mut index = self.index.lock().map_err(|_| "archive lock poisoned")?;
        let lock_path = self.safe_path(Path::new("archive.lock"))?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|e| e.to_string())?;
        lock.lock_exclusive().map_err(|e| e.to_string())?;
        let path = self.safe_path(&Self::relative_path(recording)?)?;
        fs::create_dir_all(path.parent().ok_or("archive parent missing")?)
            .map_err(|e| e.to_string())?;
        let index_path = self.safe_path(Path::new("index.jsonl"))?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(index_path)
            .map_err(|e| e.to_string())?;
        index.refresh(&mut file)?;
        if path.exists() {
            let previous = read(&path)?;
            let common = previous.tape.turns.len().min(recording.tape.turns.len());
            if previous.tape.setup != recording.tape.setup
                || previous.tape.initial_hash != recording.tape.initial_hash
                || previous.tape.turns[..common] != recording.tape.turns[..common]
            {
                return Err("game id already contains a different history; fork the recording before loading a save".into());
            }
            if previous.metadata.finished_at.is_some() && previous != *recording {
                return Err("finished archive is immutable".into());
            }
            if previous.tape.turns.len() > recording.tape.turns.len() {
                return Ok(path);
            }
        }
        // Ignore a late checkpoint after a completed game has been indexed.
        if recording.metadata.finished_at.is_none()
            && index.ids.contains(&recording.metadata.game_id)
        {
            return Ok(path);
        }
        atomic_write(&path, &bytes).map_err(|e| e.to_string())?;
        if recording.metadata.finished_at.is_some() {
            if !index.ids.contains(&recording.metadata.game_id) {
                let mut line =
                    serde_json::to_vec(&recording.metadata).map_err(|e| e.to_string())?;
                line.push(b'\n');
                file.seek(SeekFrom::End(0)).map_err(|e| e.to_string())?;
                file.write_all(&line).map_err(|e| e.to_string())?;
                file.sync_all().map_err(|e| e.to_string())?;
                index.offset += line.len() as u64;
                index.ids.insert(recording.metadata.game_id.clone());
            }
            let relative = Self::relative_path(recording)?;
            let checkpoint = self.safe_path(&Path::new("in-progress").join(relative))?;
            match fs::remove_file(checkpoint) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(path)
    }

    fn safe_path(&self, relative: &Path) -> Result<PathBuf, String> {
        let path = outside_repo(&self.root.join(relative))?;
        if !path.starts_with(&self.root) {
            return Err("archive path escapes its root".into());
        }
        Ok(path)
    }
}

impl Index {
    fn refresh(&mut self, file: &mut File) -> Result<(), String> {
        if file.metadata().map_err(|e| e.to_string())?.len() < self.offset {
            self.offset = 0;
            self.ids.clear();
        }
        file.seek(SeekFrom::Start(self.offset))
            .map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(&mut *file);
        loop {
            let mut line = Vec::new();
            let count = reader
                .read_until(b'\n', &mut line)
                .map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            // A crash during append can leave a partial last line. The immutable archive is
            // the source of truth; a retry or archive-stats --repair-index restores it.
            if !line.ends_with(b"\n") {
                break;
            }
            // Only the id matters here. Reading whole metadata made every old line written
            // under since-renamed factions ("Tollkeepers") fail the next game's commit.
            let line: serde_json::Value =
                serde_json::from_slice(&line).map_err(|e| format!("invalid index line: {e}"))?;
            let id = line["game_id"]
                .as_str()
                .ok_or("invalid index line: no game_id")?;
            self.ids.insert(id.to_owned());
            self.offset += count as u64;
        }
        file.set_len(self.offset).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("zst.tmp");
    if tmp
        .symlink_metadata()
        .is_ok_and(|m| m.file_type().is_symlink())
    {
        return Err(io::Error::other("archive temp is a symlink"));
    }
    let mut file = File::create(&tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(tmp, path)?;
    File::open(path.parent().unwrap())?.sync_all()
}
