{
    inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/*.tar.gz";
    outputs = attrs: let
        supportedSystems = [
            "aarch64-darwin"
            "x86_64-darwin"
        ];
        forEachSupportedSystem = f: attrs.nixpkgs.lib.genAttrs supportedSystems (system: f {
            inherit system;
            pkgs = import attrs.nixpkgs {
                inherit system;
            };
        });
    in {
        packages = forEachSupportedSystem ({ pkgs, system }: let
            manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
        in {
            default = pkgs.rustPlatform.buildRustPackage {
                cargoLock = {
                    allowBuiltinFetchGit = true; # allows omitting cargoLock.outputHashes
                    lockFile = ./Cargo.lock;
                };
                pname = "bitbar-mediawiki-watchlist";
                src = ./.;
                version = manifest.version;
            };
        });
    };
}
