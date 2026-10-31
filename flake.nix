{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {inherit system overlays;};
  in {
    devShells.${system}.default = pkgs.mkShell rec {
      packages = with pkgs; [
        (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        pkg-config

        clang
        lld

        stdenv.cc.cc.lib
        alsa-lib
        udev
        libxkbcommon
        libxkbcommon.dev
        wayland
        wayland-protocols
        libX11
        libXcursor
        libXrandr
        libXi
        vulkan-loader
      ];

      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath packages;
    };
  };
}
