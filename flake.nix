{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, utils, nixpkgs, fenix, }: utils.lib.eachDefaultSystem (system: let 
    pkgs = nixpkgs.legacyPackages.${system};
    rust = fenix.packages.${system};
    lib = pkgs.lib;
  in {
    devShell = pkgs.mkShell {
      buildInputs = with pkgs; with llvmPackages; [
        # For building.
        clang rust.latest.toolchain pkg-config openssl libsodium libclang.lib
      ];

      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      RUST_BACKTRACE = 1;
      # RUST_LOG = "info,sqlx::query=warn";
      RUSTFLAGS = "-C target-cpu=native";
    };
  });
}
