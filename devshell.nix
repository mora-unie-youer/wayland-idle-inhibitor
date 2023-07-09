{ pkgs, ... }:

pkgs.devShell.mkShell {
  name = "wayland-idle-inhibitor";
  packages = with pkgs; [
    # Toolchain required for C + Rust binaries building
    binutils
    gcc
    # Nightly Rust toolchain
    bacon
    (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
      # Extensions which ease your development process
      extensions = [ "rust-analyzer" "rust-src" ];
    }))
  ];
}
