use std::io::Read;

use easy_ext::ext;
use sha2::digest::Digest;

#[ext(HashExt)]
pub impl<T: Digest> T {
    fn hash(data: impl Read, buffer_size: usize) -> Vec<u8> {
        let mut hasher = T::new();
        let mut buffer = vec![0; buffer_size];
        let mut reader = data;
        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        hasher.finalize().to_vec()
    }
}
