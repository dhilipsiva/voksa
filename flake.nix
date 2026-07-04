{
  description = "voksa - rule-based Lojban Klatt-style TTS (Rust, WASM + native)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Stable rustc/cargo/clippy/rustfmt + rust-std for wasm32-unknown-unknown.
        # rust-lld ships inside the rustc component, so no external linker or
        # CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER override is needed.
        toolchain = fenix.packages.${system}.combine [
          fenix.packages.${system}.stable.toolchain
          fenix.packages.${system}.targets.wasm32-unknown-unknown.stable.rust-std
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            toolchain
            pkgs.wasm-pack
            pkgs.binaryen # wasm-opt
            pkgs.cargo-nextest
            pkgs.cargo-insta
            pkgs.twiggy
            pkgs.espeak-ng # regression oracle voice (jbo)
            pkgs.pkg-config # locates alsa.pc for cpal's alsa-sys (Phase 8)
            # Headless browser for `wasm-pack test --headless --chrome` (Phase 9).
            # nixpkgs pins chromium + chromedriver at the same version, which is
            # what wasm-bindgen-test-runner needs.
            pkgs.chromium
            pkgs.chromedriver
          ];
          # cpal links libasound on Linux; expose it via pkg-config.
          buildInputs = [ pkgs.alsa-lib ];
          # ...and on the runtime linker path so the native binary loads
          # libasound.so.2 from the nix store inside the dev shell.
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.alsa-lib ];
          # wasm-bindgen-test-runner discovers the driver + browser via these.
          CHROMEDRIVER = "${pkgs.chromedriver}/bin/chromedriver";
          CHROME_BIN = "${pkgs.chromium}/bin/chromium";
        };
      });
}
