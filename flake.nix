{
  description = "Boreas - Rust GDAL project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Native build inputs required for gdal-sys
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          clang
          llvmPackages.libclang
        ];

        # Runtime dependencies
        buildInputs = with pkgs; [
          gdal
          openssl
        ];

        # Environment variables needed for bindgen (used by gdal-sys)
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs LIBCLANG_PATH;

          shellHook = ''
            echo "Boreas development environment"
            echo "Rust version: $(rustc --version)"
            echo "GDAL version: $(gdal-config --version)"
            echo "LIBCLANG_PATH: $LIBCLANG_PATH"
          '';
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "boreas";
          version = "0.1.0";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit nativeBuildInputs buildInputs LIBCLANG_PATH;
        };
      }
    );
}