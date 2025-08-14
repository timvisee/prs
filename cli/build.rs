fn main() {
    // Warn at compiletime if no interactive selection tool is configured
    #[cfg(all(
        not(all(feature = "select-skim", unix)),
        not(feature = "select-skim-bin"),
        not(feature = "select-fzf-bin"),
    ))]
    {
        println!(
            "cargo:warning=no interactive select mode features configured, falling back to basic mode"
        );
        println!("cargo:warning=use any of: select-skim, select-skim-bin, select-fzf-bin");
    }
}
