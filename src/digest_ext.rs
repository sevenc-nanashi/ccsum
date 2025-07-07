use std::io::Read;

use duplicate::duplicate_item;
use sha2::digest::Digest;

pub trait HashExt {
    fn hash(data: impl Read, buffer_size: usize) -> Result<Vec<u8>, std::io::Error>;
}

#[duplicate_item(
    T;
    [md5::Md5];
    [sha1::Sha1];
    [sha2::Sha224];
    [sha2::Sha256];
    [sha2::Sha384];
    [sha2::Sha512];
)]
impl HashExt for T {
    fn hash(data: impl Read, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut hasher = T::new();
        let mut buffer = vec![0; buffer_size];
        let mut reader = data;
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(hasher.finalize().to_vec())
    }
}

impl HashExt for xxhash_rust::xxh3::Xxh3 {
    fn hash(data: impl Read, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
        let mut hasher = xxhash_rust::xxh3::Xxh3::new();
        let mut buffer = vec![0; buffer_size];
        let mut reader = data;
        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(hasher.digest().to_be_bytes().to_vec())
    }
}

#[duplicate_item(
    T;
    [xxhash_rust::xxh32::Xxh32];
    [xxhash_rust::xxh64::Xxh64];
)]
impl HashExt for T {
    fn hash(data: impl Read, buffer_size: usize) -> Result<Vec<u8>, std::io::Error> {
        // TODO: Allow setting seed?
        let mut hasher = T::new(0);
        let mut buffer = vec![0; buffer_size];
        let mut reader = data;
        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        Ok(hasher.digest().to_be_bytes().to_vec())
    }
}
