/* 
The main library for ice:
- [ ] first ice should parse and bare minimum deps
- [ ] ice should be capable to download files and checksums
- [ ] ice should make chroot's by hakinawa to build.
- [ ] it should be reproducible.
*/

use std::env;
use std::fs::{self, File};
use std::io::Write;
use tempfile::Builder;
use serde::Deserialize;

// this is what ICE's manifest should contain
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Manifest {
    name: String,
    ver: String,
    des: String,
    dep: Vec<String>,
    src: Vec<String>,
    hash: Vec<String>,
    cmds: String,
}

// now we start the main.
// /tmp is rw so we make a directory in /tmp
pub fn ice_init(manifest: &Manifest) -> std::io::Result<()> {
    let pth = format!("/tmp/ice-{}-{}", manifest.name, manifest.ver);
    fs::create_dir_all(&pth)?;

    let tmp = Builder::new().prefix("ice").tempdir_in(&pth)?;
    env::set_current_dir(tmp.path())?;
    Ok(())
}

pub fn ice_fetch(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    if manifest.src.len() != manifest.hash.len() {
        panic!("src and hash length mismatch");
    }

    let pth = format!("/tmp/ice-{}-{}", manifest.name, manifest.ver);
    fs::create_dir_all(&pth)?;

    let tmp_dir = Builder::new().prefix("ice-fetch").tempdir_in(&pth)?;

    for (src, _hash) in manifest.src.iter().zip(manifest.hash.iter()) {
        let response = reqwest::blocking::get(src)?;
        let fname = src
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("download.bin");

        let file_path = tmp_dir.path().join(fname);
        let mut dest = File::create(&file_path)?;
        let content = response.bytes()?;
        dest.write_all(&content)?;
        println!("downloaded '{}' to '{:?}'", fname, file_path);
    }

    Ok(())
}

pub fn ice_build() {
    // make a hakaniwa container to run bruv
}

