fn main() {
    // Crypto features warning
    #[cfg(not(any(
        feature = "backend-gnupg-bin",
        feature = "backend-gpgme",
        feature = "backend-rpgpie"
    )))]
    {
        compile_error!("no crypto backend selected, must set any of these features: backend-gnupg-bin, backend-gpgme, backend-rpgpie");
    }

    // GPG cryptography
    #[cfg(any(feature = "backend-gpgme", feature = "backend-gnupg-bin",))]
    println!("cargo:rustc-cfg=feature=\"_crypto-gpg\"");
}
