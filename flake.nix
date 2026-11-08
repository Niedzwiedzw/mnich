{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        buildInputs = with pkgs; [
          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
          just

          # wgpu, based on https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md#nix
          wayland
          libxkbcommon
          vulkan-loader
          libglvnd
          # Audio (Linux only)
          alsa-lib
          # Cross Platform 3D Graphics API
          vulkan-loader
          # For debugging around vulkan
          vulkan-tools
          # Other dependencies
          libudev-zero
          libx11
          libxcursor
          libxi
          libxrandr
        ];
        pkgConfigPath = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs;
        ldLibraryPath = pkgs.lib.makeLibraryPath buildInputs;
      in {
        devShells.default = with pkgs;
          mkShell {
            inherit buildInputs;
            PKG_CONFIG_PATH = pkgConfigPath;
            LD_LIBRARY_PATH = ldLibraryPath;
            # Set the LIBCLANG_PATH environment variable
            LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";

            shellHook = ''
            '';
          };
      }
    );
}
