use std::fs;

use eyre::bail;
use libmeow::meowzip::MeowZipMetadata;
use libmeow::{columned, ensure_superuser, meowdb};
use redb::{ReadableDatabase, ReadableTable};

use crate::install::run_hook;

pub fn remove(name: String) -> eyre::Result<()> {
    ensure_superuser()?;
    let db = meowdb::open()?;
    let read_txn = db.begin_read()?;
    let pkgs_table = read_txn.open_table(meowdb::PACKAGES)?;
    let Some(old_pkgmeta): Option<MeowZipMetadata> = pkgs_table.get(name.as_str())?.map(|row| {
        let bytes = row.value();
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .unwrap()
            .0
    }) else {
        bail!("Package `{}` is not installed", name);
    };
    let mut dependants = vec![];
    for row in pkgs_table.iter()? {
        let (depname, depmeta) = row?;
        let depmeta: MeowZipMetadata =
            bincode::decode_from_slice(depmeta.value(), bincode::config::standard())
                .unwrap()
                .0;
        if depmeta.depends.iter().any(|dep| dep == &name) {
            dependants.push(depname.value().to_owned());
        }
    }
    if !dependants.is_empty() {
        println!("The following packages depend on `{}`:", name);
        columned::print(&dependants);
        bail!(
            "Cannot remove package `{}` because other packages depend on it",
            name
        );
    }

    run_hook(
        &old_pkgmeta.name,
        &old_pkgmeta.pre_remove,
        "pre-remove",
        &old_pkgmeta.version,
        "",
    )?;

    let write_txn = db.begin_write()?;
    let mut pkgs_table = write_txn.open_table(meowdb::PACKAGES)?;
    let mut files_table = write_txn.open_table(meowdb::FILES)?;
    for oldentry in old_pkgmeta.filelist.iter().rev() {
        let meta = fs::symlink_metadata(&oldentry.filepath)?;
        if meta.is_symlink() || meta.is_file() {
            fs::remove_file(&oldentry.filepath)?;
        } else if meta.is_dir() {
            let _ = fs::remove_dir(&oldentry.filepath);
        }
        files_table.remove(&oldentry.filepath.to_str().unwrap())?;
    }
    pkgs_table.remove(name.as_str())?;

    run_hook(
        &old_pkgmeta.name,
        &old_pkgmeta.post_remove,
        "post-remove",
        &old_pkgmeta.version,
        "",
    )?;

    Ok(())
}
