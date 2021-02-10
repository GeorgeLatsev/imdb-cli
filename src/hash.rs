use sha2::{Digest, Sha256};

pub fn calculate_hash(bytes: bytes::Bytes) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(&bytes);
    let hash = sha256.finalize();

    format!("{:x}", hash)
}
