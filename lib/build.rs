fn main() {
    // GPG cryptography
    #[cfg(any(feature = "backend-gpgme", feature = "backend-gnupg-bin"))]
    println!("cargo:rustc-cfg=feature=\"_crypto-gpg\"");
}
