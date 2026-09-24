//! Staged file output for exports: write beside the destination and publish
//! only once the file is complete, so a failed export never leaves a truncated
//! file where the writer's previous export used to be.
//!
//! `tempfile` creates its files 0600, which is right for scratch space but
//! wrong for anything renamed into place: the published export would be
//! owner-only, unlike every other file in the folder. Files staged here are
//! created with the standard library defaults instead, so they get the same
//! umask-derived mode (typically 0644) as any newly created file.
use std::fs;
use std::io;
use std::path::Path;

/// The folder a destination lives in (`.` for a bare file name).
pub(crate) fn parent_dir(target: &Path) -> &Path {
    target
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

/// A uniquely named file in `dir`, removed on drop unless persisted. Created
/// with `create_new`, so it has default permissions rather than 0600.
pub(crate) fn staged_file_in(dir: &Path) -> io::Result<tempfile::NamedTempFile> {
    tempfile::Builder::new()
        .prefix(".kindling-")
        .make_in(dir, |path| {
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(path)
        })
}

/// Write `target` through a staged sibling and atomically rename it over the
/// destination only after `write` succeeds and the data is on disk. If
/// anything fails the previous file is left exactly as it was. Replacing is
/// intended here: the OS save dialog has already confirmed the overwrite. A
/// replaced file keeps its previous permissions, as an in-place rewrite would.
pub(crate) fn replace_file_atomically<F>(target: &Path, write: F) -> Result<(), String>
where
    F: FnOnce(&mut fs::File) -> Result<(), String>,
{
    let mut staged = staged_file_in(parent_dir(target))
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    if let Ok(existing) = fs::metadata(target) {
        if existing.is_file() {
            staged
                .as_file()
                .set_permissions(existing.permissions())
                .map_err(|e| format!("Failed to create output file: {}", e))?;
        }
    }
    write(staged.as_file_mut())?;
    staged
        .as_file()
        .sync_all()
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    staged.persist(target).map_err(|e| {
        format!(
            "Could not save {}; the previous file was left untouched. {}",
            target.display(),
            e.error
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn failed_write_leaves_previous_file_intact_and_no_staging() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("book.docx");
        fs::write(&target, b"previous good export").unwrap();
        let err = replace_file_atomically(&target, |file| {
            io::Write::write_all(file, b"partial").unwrap();
            Err("cover image missing".into())
        })
        .unwrap_err();
        assert_eq!(err, "cover image missing");
        assert_eq!(fs::read(&target).unwrap(), b"previous good export");
        assert_eq!(names(dir.path()), ["book.docx"]);
    }

    #[test]
    fn successful_write_replaces_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("book.docx");
        fs::write(&target, b"old").unwrap();
        replace_file_atomically(&target, |file| {
            io::Write::write_all(file, b"new").map_err(|e| e.to_string())
        })
        .unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        assert_eq!(names(dir.path()), ["book.docx"]);
    }

    /// Published output must get the mode any new file gets, not tempfile's
    /// owner-only 0600.
    #[cfg(unix)]
    #[test]
    fn staged_file_gets_default_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o7777;
        let dir = tempfile::tempdir().unwrap();

        let reference_file = dir.path().join("reference-file");
        fs::File::create(&reference_file).unwrap();
        let published_file = dir.path().join("published-file");
        replace_file_atomically(&published_file, |_| Ok(())).unwrap();
        assert_eq!(mode(&published_file), mode(&reference_file));
    }

    /// Replacing a file keeps the permissions the writer gave the old one.
    #[cfg(unix)]
    #[test]
    fn replaced_file_keeps_its_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("book.epub");
        fs::write(&target, b"old").unwrap();
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        replace_file_atomically(&target, |_| Ok(())).unwrap();
        assert_eq!(
            fs::metadata(&target).unwrap().permissions().mode() & 0o7777,
            0o640
        );
    }
}
