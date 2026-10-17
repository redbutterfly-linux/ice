use sha2::{Digest, Sha256};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn compute_file_sha256_matches_expected_digest() {
    let mut file = NamedTempFile::new().unwrap();
    let payload = b"hello from ice";
    file.write_all(payload).unwrap();

    let actual = ice::compute_file_sha256(file.path()).unwrap();
    let expected = format!(
        "{}",
        Sha256::digest(payload)
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>()
    );

    assert_eq!(actual, expected);
}
