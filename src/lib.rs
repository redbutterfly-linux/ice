/*
The main library for ice:
- [x] first ice should parse and bare minimum deps
- [x] ice should be capable to download files and checksums
- [x] ice should make chroot's by hakinawa to build.
- [ ] it should be reproducible.
*/

use hakoniwa::Container;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// Where the package workspace is mounted INSIDE the container.
const CONTAINER_WORKDIR: &str = "/build";

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

// Host-side persistent workspace for a package.
fn workspace(manifest: &Manifest) -> PathBuf {
    PathBuf::from(format!("/tmp/ice-{}-{}", manifest.name, manifest.ver))
}

// Create a persistent workspace for this package.
pub fn ice_init(manifest: &Manifest) -> io::Result<()> {
    let pth = workspace(manifest);
    fs::create_dir_all(&pth)?;
    env::set_current_dir(&pth)?;
    Ok(())
}

// this is a misc function.
pub fn compute_file_sha256<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>())
}

// download the sources required for ice.
pub fn ice_fetch(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    if manifest.src.len() != manifest.hash.len() {
        return Err("src and hash length mismatch".into());
    }

    let tmp_dir = workspace(manifest);
    fs::create_dir_all(&tmp_dir)?;

    for (src, expected_hash) in manifest.src.iter().zip(manifest.hash.iter()) {
        let response = reqwest::blocking::get(src)?.error_for_status()?;
        let fname = src
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("download.bin");

        let file_path = tmp_dir.join(fname);
        let mut dest = File::create(&file_path)?;
        let content = response.bytes()?;
        dest.write_all(&content)?;
        println!("downloaded '{}' to '{}'", fname, file_path.display());

        if expected_hash != "SKIP" {
            let rhash = compute_file_sha256(&file_path)?;
            if rhash != expected_hash.to_ascii_lowercase() {
                return Err(format!("SHA-256 mismatch for '{}'", fname).into());
            }
        }
    }

    Ok(())
}

pub fn ice_build(manifest: &Manifest) -> Result<(), Box<dyn std::error::Error>> {
    let host_dir = workspace(manifest);
    fs::create_dir_all(&host_dir)?;

    // The script is written on the host, but runs inside the container,
    // so every path in it must be the *container* path (/build).
    let host_script = host_dir.join("build.sh");
    let script = format!(
        "#!/bin/sh\nset -eu\ncd '{wd}'\nexport DESTDIR='{wd}/destdir'\n{cmds}\n",
        wd = CONTAINER_WORKDIR,
        cmds = manifest.cmds
    );
    fs::write(&host_script, script)?;

    let mut perms = fs::metadata(&host_script)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&host_script, perms)?;

    let host_dir_str = host_dir.to_str().ok_or("invalid workspace path")?;
    let container_script = format!("{}/build.sh", CONTAINER_WORKDIR);

    // rootfs("/") already mounts /bin /etc /lib /lib64 /lib32 /sbin /usr
    // read-only. Do NOT bind /usr again, and do NOT bind the whole host /tmp.
    let mut container = Container::new();
    container
        .rootfs("/")?
        .tmpfsmount("/tmp") // private scratch /tmp for gcc temp files etc.
        .bindmount_rw(host_dir_str, CONTAINER_WORKDIR); // the only writable host dir

    let status = container
        .command("/bin/sh")
        .arg(&container_script)
        // The container env is not inherited; gcc needs PATH to locate cc1.
        .env("PATH", "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin")
        .env("HOME", "/tmp")
        .env("TMPDIR", "/tmp")
        .env("LANG", "C")
        .status()
        .expect("failed to execute process within container");

    // Always show the result; a setup failure shows up here, not as an Err.
    println!("[ICE] container status: {:?}", status);

    if !status.success() {
        return Err(format!("build failed: {:?}", status).into());
    }

    Ok(())
}