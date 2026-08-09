{
  description = "Hermes build environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        sql = ./hermes.sql;
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.cmake     # builds aws-lc-sys (jsonwebtoken -> aws_lc_rs)
            pkgs.perl      # aws-lc-sys codegen scripts
            pkgs.clang     # libclang, used by aws-lc-sys's bindgen fallback
            pkgs.nasm      # lets aws-lc-sys use optimized asm on x86_64
          ];

          buildInputs = [
            pkgs.openssl
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          shellHook = ''
            echo "===== nuking db ====="
            sudo -u postgres psql -d records_ps -c "DROP SCHEMA public CASCADE;CREATE SCHEMA public;"
            echo "===== recreating db ====="
            sudo -u postgres psql -d records_ps -f ${sql}
            echo "===== db recreated ====="
          '';
        };
      });
}