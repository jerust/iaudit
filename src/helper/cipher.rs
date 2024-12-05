use murmurhash64::murmur_hash64a;

const SEED: u64 = 2915580697;

pub fn murmurhash64int<S: AsRef<str>>(s: S) -> u64 {
    murmur_hash64a(s.as_ref().as_bytes(), SEED)
}

pub fn murmurhash64str<S: AsRef<str>>(s: S) -> String {
    murmurhash64int(s).to_string()
}
