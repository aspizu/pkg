use std::fs;
use std::path::{Path, PathBuf};

use eyre::bail;
use libmeow::meowzip::MeowZipMetadata;
use libmeow::{columned, ensure_superuser, meowdb};
use redb::{ReadableDatabase, ReadableTable, Table};

use crate::install::run_hook;

pub fn remove(name: String, root: PathBuf) -> eyre::Result<()> {
    ensure_superuser()?;
    let db = meowdb::open(&root)?;
    let read_txn = db.begin_read()?;
    let packages = read_txn.open_table(meowdb::PACKAGES)?;
    let Some(pkgmeta): Option<MeowZipMetadata> =
        packages.get(name.as_str())?.map(|row| row.value().into())
    else {
        bail!("Package `{}` is not installed", name);
    };

    let mut dependants = vec![];
    for row in packages.iter()? {
        let (depname, depmeta) = row?;
        let depmeta = MeowZipMetadata::from(depmeta.value());
        if depmeta.depends.iter().any(|dep| dep == &name) {
            dependants.push(depname.value().to_owned());
        }
    }
    if !dependants.is_empty() {
        println!("The following packages depend on `{}`:", name);
        columned::print(&dependants);
        bail!("Cannot remove package `{}` because other packages depend on it", name);
    }

    if &root == "" {
        run_hook(&pkgmeta.name, &pkgmeta.pre_remove, "pre-remove", &pkgmeta.version, "")?;
    }

    let write_txn = db.begin_write()?;
    let mut pkgs_table = write_txn.open_table(meowdb::PACKAGES)?;
    let mut files_table = write_txn.open_table(meowdb::FILES)?;
    for entry in pkgmeta.filelist.iter().rev() {
        uninstall_path(&root, &entry.filepath, &mut files_table)?;
    }
    pkgs_table.remove(&*name)?;

    if &root == "" {
        run_hook(&pkgmeta.name, &pkgmeta.post_remove, "post-remove", &pkgmeta.version, "")?;
    }

    Ok(())
}

pub fn uninstall_path(
    root: &Path,
    path: &Path,
    files_table: &mut Table<&str, &[u8]>,
) -> eyre::Result<()> {
    let dest = root.join(path);
    if fs::exists(&dest)? {
        let meta = fs::symlink_metadata(&dest)?;
        if meta.is_symlink() || meta.is_file() {
            fs::remove_file(&dest)?;
        } else if meta.is_dir() {
            let _ = fs::remove_dir(dest);
        }
    }
    files_table.remove(path.to_str().unwrap())?;
    Ok(())
}
