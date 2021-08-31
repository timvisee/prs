fn main() {
    // Crypto features warning
    #[cfg(not(any(feature = "backend-gnupg-bin", feature = "backend-gpgme")))]
    {
        println!("cargo:warning=prs-lib: no crypto backend selected, you must set any of the following features: backend-gnupg-bin, backend-gpgme");
    }

    // GPG cryptography
    #[cfg(any(feature = "backend-gpgme", feature = "backend-gnupg-bin"))]
    println!("cargo:rustc-cfg=feature=\"_crypto-gpg\"");
}
