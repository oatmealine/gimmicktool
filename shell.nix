# Run with `nix-shell shell.nix`
let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    pkg-config
    wrapGAppsHook4
    cargo 
    cargo-tauri # Optional, Only needed if Tauri doesn't work through the traditional way.
    clippy
    nodejs # Optional, this is for if you have a js frontend
    rustc # Needed for dev server (npm tauri dev)
    pnpm
  ];

  buildInputs = with pkgs; [
    librsvg
    webkitgtk_4_1
  ];

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH" # Needed on Wayland to report the correct display scale
  '';
}