mod platform;

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

pub type Result<T> = std::result::Result<T, String>;
const LIMIT: u64 = 16 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
pub struct FontFile {
    pub filename: String,
    pub sha256: String,
    pub size: u64,
    pub weight: u16,
    pub italic: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub license: String,
    pub files: Vec<FontFile>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub revision: String,
    pub families: Vec<Family>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Receipt {
    family: String,
    filename: String,
    sha256: String,
}
#[derive(Default, Serialize, Deserialize)]
struct Receipts {
    files: Vec<Receipt>,
}
#[derive(Serialize)]
pub struct Status {
    pub id: String,
    pub installed: usize,
    pub total: usize,
    pub changed: usize,
    pub managed: usize,
}
#[derive(Serialize)]
pub struct Outcome {
    pub warning: String,
    pub count: usize,
}

pub fn catalog() -> Result<Catalog> {
    let catalog: Catalog = serde_json::from_str(include_str!("../catalog.json")).map_err(err)?;
    if catalog.revision.len() != 40 || !catalog.revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Invalid source revision".into());
    }
    for family in &catalog.families {
        if family.id.is_empty()
            || !family
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err("Invalid catalog family".into());
        }
        for font in &family.files {
            if !safe_filename(&font.filename)
                || font.sha256.len() != 64
                || !font.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                || font.size > LIMIT
            {
                return Err("Invalid catalog font".into());
            }
        }
    }
    Ok(catalog)
}
fn safe_filename(s: &str) -> bool {
    !s.is_empty()
        && !s.contains(['/', '\\', '\0', ':'])
        && !s.starts_with('.')
        && s.ends_with(".ttf")
}
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn validate_font(data: &[u8], font: &FontFile) -> Result<()> {
    if data.len() as u64 != font.size || digest(data) != font.sha256 {
        return Err("Font checksum mismatch; download rejected".into());
    }
    let face = ttf_parser::Face::parse(data, 0).map_err(err)?;
    // Thai consonants, above/below marks and tone marks must all be present.
    if ['ก', 'ำ', 'ิ', 'ุ', '่', '้', '๙']
        .iter()
        .any(|c| face.glyph_index(*c).is_none())
    {
        return Err("Font does not contain the required Thai characters".into());
    }
    Ok(())
}

pub trait Platform {
    fn register(&self, path: &Path, add: bool) -> Result<()>;
    fn refresh(&self, dir: &Path) -> String;
}
pub struct Native;
impl Platform for Native {
    fn register(&self, path: &Path, add: bool) -> Result<()> {
        platform::register(path, add)
    }
    fn refresh(&self, dir: &Path) -> String {
        platform::refresh(dir)
    }
}
pub struct Manager<P: Platform = Native> {
    data: PathBuf,
    fonts: PathBuf,
    platform: P,
    _lock: File,
}

impl Manager<Native> {
    pub fn open() -> Result<Self> {
        let dirs = directories::ProjectDirs::from("com", "ChiStudio", "ThaiFontKit")
            .ok_or("Cannot resolve application directory")?;
        Self::at(
            dirs.data_local_dir().to_owned(),
            platform::font_dir()?,
            Native,
        )
    }
}
impl<P: Platform> Manager<P> {
    pub fn at(data: PathBuf, fonts: PathBuf, platform: P) -> Result<Self> {
        fs::create_dir_all(&data).map_err(err)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(data.join("app.lock"))
            .map_err(err)?;
        lock.try_lock_exclusive().map_err(|_| {
            "Thai Font Kit is busy in another window. Please try again.".to_string()
        })?;
        Ok(Self {
            data,
            fonts,
            platform,
            _lock: lock,
        })
    }
    fn receipts(&self) -> Result<Receipts> {
        match fs::read(self.data.join("installed.json")) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|_| "Installation record is damaged. No fonts were changed.".into()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Receipts::default()),
            Err(e) => Err(err(e)),
        }
    }
    fn save(&self, receipts: &Receipts) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(receipts).map_err(err)?;
        let mut temp = tempfile::NamedTempFile::new_in(&self.data).map_err(err)?;
        temp.write_all(&bytes).map_err(err)?;
        temp.as_file().sync_all().map_err(err)?;
        temp.persist(self.data.join("installed.json"))
            .map_err(err)?;
        Ok(())
    }
    fn family(&self, id: &str) -> Result<(Catalog, Family)> {
        let cat = catalog()?;
        let family = cat
            .families
            .iter()
            .find(|f| f.id == id)
            .cloned()
            .ok_or("Unknown font family")?;
        Ok((cat, family))
    }
    fn destination(&self, id: &str, font: &FontFile) -> PathBuf {
        self.fonts
            .join(format!("ThaiFontKit-{id}-{}", font.filename))
    }
    fn owned(&self, receipt: &Receipt) -> Result<PathBuf> {
        let (_, family) = self.family(&receipt.family)?;
        let font = family
            .files
            .iter()
            .find(|f| f.filename == receipt.filename && f.sha256 == receipt.sha256)
            .ok_or("Unrecognized installation record. No files were removed.")?;
        Ok(self.destination(&family.id, font))
    }
    fn matches(path: &Path, expected: &str) -> bool {
        if fs::symlink_metadata(path)
            .map(|m| m.file_type().is_symlink() || !m.is_file() || m.len() > LIMIT)
            .unwrap_or(true)
        {
            return false;
        }
        fs::read(path)
            .map(|bytes| bytes.len() as u64 <= LIMIT && digest(&bytes) == expected)
            .unwrap_or(false)
    }
    pub fn status(&self) -> Result<Vec<Status>> {
        let receipts = self.receipts()?;
        catalog()?
            .families
            .iter()
            .map(|family| {
                let mut installed = 0;
                let mut changed = 0;
                for receipt in receipts.files.iter().filter(|r| r.family == family.id) {
                    let font = family
                        .files
                        .iter()
                        .find(|f| f.filename == receipt.filename && f.sha256 == receipt.sha256)
                        .ok_or("Unrecognized installation record. No files were changed.")?;
                    let path = self.destination(&family.id, font);
                    if Self::matches(&path, &receipt.sha256) {
                        installed += 1;
                    } else if path.symlink_metadata().is_ok() {
                        changed += 1;
                    }
                }
                let managed = receipts
                    .files
                    .iter()
                    .filter(|r| r.family == family.id)
                    .count();
                Ok(Status {
                    id: family.id.clone(),
                    installed,
                    total: family.files.len(),
                    changed,
                    managed,
                })
            })
            .collect()
    }
    pub fn font_bytes(&self, id: &str) -> Result<Vec<u8>> {
        let (cat, family) = self.family(id)?;
        self.download(
            &cat.revision,
            &family.id,
            family
                .files
                .iter()
                .min_by_key(|f| (f.italic, f.weight.abs_diff(400)))
                .ok_or("Empty font family")?,
        )
    }
    fn download(&self, revision: &str, id: &str, font: &FontFile) -> Result<Vec<u8>> {
        let installed = self.destination(id, font);
        if Self::matches(&installed, &font.sha256) {
            return fs::read(installed).map_err(err);
        }
        let cache_dir = self.data.join("cache");
        fs::create_dir_all(&cache_dir).map_err(err)?;
        let cached = cache_dir.join(format!("{}.ttf", font.sha256));
        if let Ok(data) = fs::read(&cached) {
            if validate_font(&data, font).is_ok() {
                return Ok(data);
            }
        }
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("ThaiFontKit/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(err)?;
        let mut url = reqwest::Url::parse(&format!(
            "https://raw.githubusercontent.com/google/fonts/{revision}/ofl/{id}/"
        ))
        .map_err(err)?;
        url.path_segments_mut()
            .map_err(|_| "Invalid font URL")?
            .pop_if_empty()
            .push(&font.filename);
        let response = client
            .get(url)
            .send()
            .map_err(|e| format!("Cannot download font. Check your connection. {e}"))?
            .error_for_status()
            .map_err(err)?;
        if response.content_length().unwrap_or(0) > LIMIT {
            return Err("Font file is too large".into());
        }
        let mut data = Vec::new();
        response
            .take(LIMIT + 1)
            .read_to_end(&mut data)
            .map_err(err)?;
        validate_font(&data, font)?;
        let mut temp = tempfile::NamedTempFile::new_in(&cache_dir).map_err(err)?;
        temp.write_all(&data).map_err(err)?;
        temp.persist(&cached).map_err(err)?;
        Ok(data)
    }
    pub fn install(&self, id: &str) -> Result<Outcome> {
        self.install_with_progress(id, |_, _, _| {})
    }
    pub fn install_with_progress(
        &self,
        id: &str,
        progress: impl Fn(&str, usize, usize),
    ) -> Result<Outcome> {
        let (cat, family) = self.family(id)?;
        let mut receipts = self.receipts()?;
        // Validate all downloads before touching the system fonts directory.
        let files: Vec<_> = family
            .files
            .iter()
            .enumerate()
            .map(|(index, font)| {
                progress("download", index, family.files.len());
                self.download(&cat.revision, id, font)
                    .map(|bytes| (font, bytes))
            })
            .collect::<Result<_>>()?;
        fs::create_dir_all(&self.fonts).map_err(err)?;
        let mut count = 0;
        for (index, (font, bytes)) in files.into_iter().enumerate() {
            progress("install", index, family.files.len());
            let path = self.destination(id, font);
            let owned = receipts
                .files
                .iter()
                .any(|r| r.family == id && r.filename == font.filename && r.sha256 == font.sha256);
            if path.symlink_metadata().is_ok() {
                if owned && Self::matches(&path, &font.sha256) {
                    self.platform.register(&path, true)?;
                    continue;
                }
                return Err(format!("A file already exists: {}. It was not overwritten. Some other styles may already be installed; retry after resolving the conflict.", path.display()));
            }
            // Journal before creation: an interrupted install remains discoverable and removable.
            if !owned {
                receipts.files.push(Receipt {
                    family: id.into(),
                    filename: font.filename.clone(),
                    sha256: font.sha256.clone(),
                });
                self.save(&receipts)?;
            }
            let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => file,
                Err(e) => {
                    receipts
                        .files
                        .retain(|r| !(r.family == id && r.filename == font.filename));
                    self.save(&receipts)?;
                    return Err(err(e));
                }
            };
            if let Err(e) = file.write_all(&bytes).and_then(|_| file.sync_all()) {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err(format!(
                    "Cannot write font: {e}. Retry to finish installation."
                ));
            }
            drop(file);
            if let Err(e) = self.platform.register(&path, true) {
                // Keep the receipt until deletion succeeds; retries can recover it.
                fs::remove_file(&path).map_err(err)?;
                return Err(e);
            }
            count += 1;
        }
        // Preserve licence alongside receipts, outside the system font directory.
        fs::write(self.data.join(format!("OFL-{id}.txt")), &family.license).map_err(err)?;
        // Installed files are the verified source for future previews; remove duplicate downloads.
        for font in &family.files {
            let _ = fs::remove_file(self.data.join("cache").join(format!("{}.ttf", font.sha256)));
        }
        progress("refresh", family.files.len(), family.files.len());
        Ok(Outcome {
            count,
            warning: self.platform.refresh(&self.fonts),
        })
    }
    pub fn uninstall(&self, id: &str) -> Result<Outcome> {
        self.family(id)?;
        let mut receipts = self.receipts()?;
        let selected: Vec<_> = receipts
            .files
            .iter()
            .filter(|r| r.family == id)
            .cloned()
            .collect();
        let mut count = 0;
        for receipt in selected {
            let path = self.owned(&receipt)?;
            if path.symlink_metadata().is_ok() {
                if !Self::matches(&path, &receipt.sha256) {
                    return Err(format!(
                        "{} has changed. It was not removed.",
                        path.display()
                    ));
                }
                self.platform.register(&path, false)?;
                fs::remove_file(&path)
                    .map_err(|e| format!("Close apps using this font and try again: {e}"))?;
                count += 1;
            } else {
                // Clean persistent Windows/macOS registrations after a manually deleted file.
                self.platform.register(&path, false)?;
            }
            receipts
                .files
                .retain(|r| !(r.family == receipt.family && r.filename == receipt.filename));
            self.save(&receipts)?;
        }
        Ok(Outcome {
            count,
            warning: self.platform.refresh(&self.fonts),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake;
    impl Platform for Fake {
        fn register(&self, _: &Path, _: bool) -> Result<()> {
            Ok(())
        }
        fn refresh(&self, _: &Path) -> String {
            String::new()
        }
    }
    #[test]
    fn verified_cached_font_installs_offline_and_uninstalls_cleanly() {
        let tmp = tempfile::tempdir().unwrap();
        let m = Manager::at(tmp.path().join("data"), tmp.path().join("fonts"), Fake).unwrap();
        let family = catalog()
            .unwrap()
            .families
            .into_iter()
            .find(|f| f.id == "anuphan")
            .unwrap();
        assert_eq!(family.files.len(), 1);
        let font = &family.files[0];
        let bytes = include_bytes!("../assets/Anuphan.ttf");
        validate_font(bytes, font).unwrap();
        fs::create_dir_all(m.data.join("cache")).unwrap();
        fs::write(
            m.data.join("cache").join(format!("{}.ttf", font.sha256)),
            bytes,
        )
        .unwrap();
        let events = std::cell::RefCell::new(Vec::new());
        assert_eq!(
            m.install_with_progress("anuphan", |phase, done, total| {
                events.borrow_mut().push((phase.to_string(), done, total));
            })
            .unwrap()
            .count,
            1
        );
        assert_eq!(
            *events.borrow(),
            vec![
                ("download".into(), 0, 1),
                ("install".into(), 0, 1),
                ("refresh".into(), 1, 1)
            ]
        );
        assert!(!m
            .data
            .join("cache")
            .join(format!("{}.ttf", font.sha256))
            .exists());
        assert_eq!(m.font_bytes("anuphan").unwrap(), bytes.to_vec());
        assert_eq!(m.install("anuphan").unwrap().count, 0);
        assert_eq!(
            m.status()
                .unwrap()
                .into_iter()
                .find(|s| s.id == "anuphan")
                .unwrap()
                .installed,
            1
        );
        assert!(m.data.join("OFL-anuphan.txt").exists());
        fs::write(m.fonts.join("personal.ttf"), b"keep").unwrap();
        assert_eq!(m.uninstall("anuphan").unwrap().count, 1);
        assert!(m.fonts.join("personal.ttf").exists());
        assert!(m.receipts().unwrap().files.is_empty());
    }
    #[test]
    fn matching_unmanaged_font_is_not_adopted_or_deleted() {
        let tmp = tempfile::tempdir().unwrap();
        let m = Manager::at(tmp.path().join("data"), tmp.path().join("fonts"), Fake).unwrap();
        let family = catalog()
            .unwrap()
            .families
            .into_iter()
            .find(|f| f.id == "anuphan")
            .unwrap();
        let font = &family.files[0];
        let bytes = include_bytes!("../assets/Anuphan.ttf");
        fs::create_dir_all(m.data.join("cache")).unwrap();
        fs::create_dir_all(&m.fonts).unwrap();
        fs::write(
            m.data.join("cache").join(format!("{}.ttf", font.sha256)),
            bytes,
        )
        .unwrap();
        let path = m.destination("anuphan", font);
        fs::write(&path, bytes).unwrap();
        assert!(m.install("anuphan").is_err());
        assert!(m.receipts().unwrap().files.is_empty());
        assert_eq!(m.uninstall("anuphan").unwrap().count, 0);
        assert!(path.exists());
    }
    #[test]
    fn catalog_is_valid_and_has_thai_families() {
        let cat = catalog().unwrap();
        assert_eq!(cat.families.len(), 34);
        for f in cat.families {
            assert!(!f.files.is_empty());
            assert!(f.license.contains("SIL OPEN FONT LICENSE"));
        }
    }
    #[test]
    fn rejects_paths_and_tampered_downloads() {
        for name in ["../x.ttf", "C:\\x.ttf", ".x.ttf", "/x.ttf", "x:foo.ttf"] {
            assert!(!safe_filename(name));
        }
        let f = catalog().unwrap().families.remove(0).files.remove(0);
        assert!(validate_font(b"untrusted", &f).is_err());
    }
    #[test]
    fn legacy_installed_fonts_remain_recognizable_after_upgrade() {
        let old: Catalog = serde_json::from_str(include_str!("../legacy-v0.1.json")).unwrap();
        let current = catalog().unwrap();
        for old_family in old.families {
            let family = current
                .families
                .iter()
                .find(|f| f.id == old_family.id)
                .expect("Legacy family was removed");
            for old_font in old_family.files {
                let font = family
                    .files
                    .iter()
                    .find(|f| f.filename == old_font.filename)
                    .expect("Legacy font was removed");
                assert_eq!(
                    font.sha256, old_font.sha256,
                    "Existing installation receipts must stay valid"
                );
            }
        }
    }
    #[test]
    #[ignore = "maintainer check: requires TFK_FONT_CACHE populated by scripts/catalog.py"]
    fn verify_all_catalog_fonts() {
        let cache = PathBuf::from(std::env::var("TFK_FONT_CACHE").expect("Set TFK_FONT_CACHE"));
        let cat = catalog().unwrap();
        let mut count = 0;
        for family in cat.families {
            for font in family.files {
                let data = fs::read(cache.join(format!("{}.ttf", font.sha256))).unwrap();
                validate_font(&data, &font)
                    .unwrap_or_else(|e| panic!("{} / {}: {e}", family.name, font.filename));
                count += 1;
            }
        }
        println!("Verified SHA-256, font parsing and Thai coverage for {count} font files");
    }
    #[test]
    fn uninstall_preserves_modified_and_unmanaged_fonts() {
        let tmp = tempfile::tempdir().unwrap();
        let m = Manager::at(tmp.path().join("data"), tmp.path().join("fonts"), Fake).unwrap();
        fs::create_dir_all(&m.fonts).unwrap();
        let f = catalog().unwrap().families.remove(0);
        let font = &f.files[0];
        let path = m.destination(&f.id, font);
        fs::write(&path, b"user-modified").unwrap();
        fs::write(m.fonts.join("personal.ttf"), b"keep").unwrap();
        m.save(&Receipts {
            files: vec![Receipt {
                family: f.id.clone(),
                filename: font.filename.clone(),
                sha256: font.sha256.clone(),
            }],
        })
        .unwrap();
        assert!(m.uninstall(&f.id).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"user-modified");
        assert!(m.fonts.join("personal.ttf").exists());
    }
    #[test]
    fn forged_receipt_cannot_escape_catalog() {
        let tmp = tempfile::tempdir().unwrap();
        let m = Manager::at(tmp.path().join("data"), tmp.path().join("fonts"), Fake).unwrap();
        m.save(&Receipts {
            files: vec![Receipt {
                family: "sarabun".into(),
                filename: "../../personal.ttf".into(),
                sha256: "x".into(),
            }],
        })
        .unwrap();
        assert!(m.uninstall("sarabun").is_err());
    }
    #[test]
    fn missing_files_can_be_cleaned_and_lock_blocks_second_instance() {
        let tmp = tempfile::tempdir().unwrap();
        let data = tmp.path().join("data");
        let fonts = tmp.path().join("fonts");
        let m = Manager::at(data.clone(), fonts.clone(), Fake).unwrap();
        assert!(Manager::at(data, fonts, Fake).is_err());
        let f = catalog().unwrap().families.remove(0);
        let font = &f.files[0];
        m.save(&Receipts {
            files: vec![Receipt {
                family: f.id.clone(),
                filename: font.filename.clone(),
                sha256: font.sha256.clone(),
            }],
        })
        .unwrap();
        assert_eq!(m.uninstall(&f.id).unwrap().count, 0);
        assert!(m.receipts().unwrap().files.is_empty());
    }
    #[cfg(unix)]
    #[test]
    fn uninstall_refuses_symlinks() {
        use std::os::unix::fs::symlink;
        let tmp = tempfile::tempdir().unwrap();
        let m = Manager::at(tmp.path().join("data"), tmp.path().join("fonts"), Fake).unwrap();
        fs::create_dir_all(&m.fonts).unwrap();
        let f = catalog().unwrap().families.remove(0);
        let font = &f.files[0];
        let outside = tmp.path().join("personal.ttf");
        fs::write(&outside, b"keep").unwrap();
        symlink(&outside, m.destination(&f.id, font)).unwrap();
        m.save(&Receipts {
            files: vec![Receipt {
                family: f.id.clone(),
                filename: font.filename.clone(),
                sha256: font.sha256.clone(),
            }],
        })
        .unwrap();
        assert!(m.uninstall(&f.id).is_err());
        assert!(outside.exists());
    }
}
