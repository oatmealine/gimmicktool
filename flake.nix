{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
  };

  outputs = { self, nixpkgs }:
    let
      forAllSystems = function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ] (system: function {
          inherit system;
          pkgs = import nixpkgs {
            inherit system;
          };
          lib = nixpkgs.lib;
        });
    in {
      packages = forAllSystems ({ pkgs, lib, ... }: rec {
        gimmicktool = let
          package = builtins.fromJSON (builtins.readFile ./package.json);
          pnpm = pkgs.pnpm_10;
        in pkgs.rustPlatform.buildRustPackage (final: {
          pname = "gimmicktool";
          inherit (package) version;
          src = ./.;
          
          cargoHash = "sha256-rmO2bJdkzamhyju7XMPw0Q+WFrsJCiq0EqNZbWoRymw=";

          pnpmDeps = (pkgs.fetchPnpmDeps {
            inherit (final) pname version src;
            inherit pnpm;
            hash = "sha256-HvLwbS63mzhfmsVaD4H5Qkw7x6XMgE0s7CDGi+syKA4=";
            fetcherVersion = 3;
          });

          nativeBuildInputs = with pkgs; [
            cargo-tauri.hook
            nodejs
            pnpm
            pnpmConfigHook
            pkg-config
          ]
          ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ];

          buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isLinux (with pkgs; [
            #glib-networking # Most Tauri apps need networking
            openssl
            webkitgtk_4_1
          ]);

          cargoRoot = "src-tauri";
          buildAndTestSubdir = final.cargoRoot;
        });
        default = gimmicktool;
      });
      devShells = forAllSystems ({ pkgs, ... }: {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            cargo 
            #cargo-tauri # Optional, Only needed if Tauri doesn't work through the traditional way.
            clippy
            nodejs
            rustc
            pnpm
          ]
          ++ lib.optionals stdenv.hostPlatform.isLinux [ wrapGAppsHook4 ];

          buildInputs = with pkgs; [
            librsvg
            webkitgtk_4_1
          ];

          shellHook = ''
            export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH" # Needed on Wayland to report the correct display scale
          '';
        };
      });
    };
}
