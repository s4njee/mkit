use crate::{Result, support::atomic_write};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};
use toml_edit::{ArrayOfTables, DocumentMut, Item, Table, value};
#[derive(Debug, Deserialize)]
pub(crate) struct LockFile {
    #[serde(default)]
    pub(crate) components: Vec<LockedComponent>,
}
#[derive(Debug, Deserialize)]
pub(crate) struct LockedComponent {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) origin: String,
    #[serde(default)]
    pub(crate) files: BTreeMap<String, LockedFile>,
}
#[derive(Debug, Deserialize)]
pub(crate) struct LockedFile {
    pub(crate) base: String,
    pub(crate) sha256: String,
}

pub(crate) fn write_lock(doc: &mut DocumentMut, lock: &LockFile, path: &Path) -> Result<()> {
    let mut arr = ArrayOfTables::new();
    for comp in &lock.components {
        let mut t = Table::new();
        t["name"] = value(&comp.name);
        t["version"] = value(&comp.version);
        t["origin"] = value(&comp.origin);
        let mut files = Table::new();
        for (installed, file) in &comp.files {
            let mut ft = Table::new();
            ft["base"] = value(&file.base);
            ft["sha256"] = value(&file.sha256);
            files[installed] = Item::Table(ft);
        }
        t["files"] = Item::Table(files);
        arr.push(t);
    }
    doc["components"] = Item::ArrayOfTables(arr);
    atomic_write(path, doc.to_string().as_bytes())
}

pub(crate) fn safe_relative(s: &str) -> Result<()> {
    let p = Path::new(s);
    if p.as_os_str().is_empty()
        || p.is_absolute()
        || p.components().any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path `{s}`").into());
    }
    Ok(())
}
pub(crate) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
