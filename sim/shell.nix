{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
  buildInputs = with pkgs; [
    wayland
    #kdePackages.wayland
    udev
    alsa-lib
    
    
    #libxkbcommon
  ];
  # Ensure dlopen finds the libraries at runtime
  LD_LIBRARY_PATH = with pkgs; "${lib.makeLibraryPath [ libxkbcommon vulkan-loader ]}";
}
