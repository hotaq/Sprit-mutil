{
  description = "Sprite Multi-Agent Workflow Toolkit";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        buildInputs = with pkgs; [
          tmux
          git
          openssl
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.Security
        ];

        sprite = pkgs.rustPlatform.buildRustPackage {
          pname = "sprite";
          version = "0.1.0";

          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          inherit nativeBuildInputs buildInputs;

          meta = with pkgs.lib; {
            description = "A robust command-line toolkit for managing multiple AI coding agents in isolated tmux sessions";
            homepage = "https://github.com/hotaq/Sprit-mutil";
            license = licenses.mit;
            maintainers = with maintainers; [ hotaq ];
            platforms = platforms.unix;
            mainProgram = "sprite";
          };
        };
      in
      {
        packages.default = sprite;

        apps.default = flake-utils.lib.mkApp {
          drv = sprite;
        };

        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          shellHook = ''
            export RUST_LOG=debug
            echo "🚀 Sprite Development Environment"
            echo "Run 'cargo run -- --help' to test the CLI"
          '';
        };
      });
}