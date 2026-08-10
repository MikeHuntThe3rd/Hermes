{
  description = "Hermes build environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
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
            pkgs.cmake
            pkgs.perl
            pkgs.clang
            pkgs.nasm
          ];

          buildInputs = [
            pkgs.openssl
          ];

          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        devShells.setup = pkgs.mkShell {
          shellHook = ''
            echo "===== setting up db schema ====="
            sudo -u postgres psql -c "CREATE ROLE root WITH LOGIN;"
            sudo -u postgres psql -c "CREATE DATABASE records_ps OWNER root;"
            sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE records_ps TO root;"
          '';
        };

        devShells.reset = pkgs.mkShell {
          shellHook = ''
            echo "===== nuking db ====="
            sudo -u postgres psql -d records_ps -c "DROP SCHEMA public CASCADE;CREATE SCHEMA public;"
            echo "===== recreating db ====="
            sudo -u postgres psql -d records_ps -f ${sql}
            echo "===== db recreated ====="
            sudo -u postgres psql -d records_ps -c "
            GRANT USAGE ON SCHEMA public TO root;
            GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO root;
            GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO root;"
            echo "===== privileges granted ====="
          '';
        };
      }
    );
}