/* 
The main library for ice:
- [ ] first ice should parse and bare minimum deps
- [ ] ice should be capable to download files and checksums
- [ ] ice should make chroot's by hakinawa to build.
- [ ] it should be reproducible.
*/

use mktemp::Temp;
use std::fs;
use serde::Deserialize;

// this is what ICE's manifest should contain
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
pub fn ice_init()
{
    let bldpath = Temp::new_;
}