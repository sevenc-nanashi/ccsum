use easy_ext::ext;
use sha2::digest::Digest;

#[ext(HashExt)]
pub impl<T: Digest> T {
    fn hash(data: impl AsRef<[u8]>) -> Vec<u8> {
        let mut hasher = T::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }
}
