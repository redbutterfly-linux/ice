/* 
The main library for ice:
- [ ] first ice should parse and bare minimum deps
- [ ] ice should be capable to download files and checksums
- [ ] ice should make chroot's by hakinawa to build.
- [ ] it should be reproducible.
*/

use mktemp::Temp;
use std::env;
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
    cmds: String
}

// now we start the main.
// /tmp is rw so we make a directory in /tmp
pub fn ice_init(manifest: &Manifest) {
    let pth = format!("/tmp/ice-{}-{}", manifest.name, manifest.ver);
    let tmp = Temp::new_dir_in(pth).unwrap();
    env::set_current_dir(tmp.as_ref()).unwrap();
}

pub fn ice_fetch(manifest: &Manifest) {
    if (manifest.src.len() != manifest.hash.len()) {
        panic!("src and hash length mismatch");
    }


}

pub fn ice_build() {
   // make a hakaniwa container to run bruv
}

