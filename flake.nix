{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs = [
              cargo
              rustc
              rustfmt
              pre-commit
              rustPackages.clippy
              cmake
              raylib
              glfw
              pkg-config
              clang
              libglvnd
              #X11 deps
              xorg.libX11
              xorg.libX11.dev
              xorg.libXcursor
              xorg.libXi
              xorg.libXinerama
              xorg.libXrandr
              xorg.libXft
              expat
              rustup
            ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;

            shellHook = ''
              export LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib";
            '';
            LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
            LD_LIBRARY_PATH = "${libglvnd}/lib";
          };

      });
}
