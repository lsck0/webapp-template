# https://nixos.wiki/wiki/Development_environment_with_nix-shell
# https://search.nixos.org/packages
{
  description = "Webapp Template Flake";

  inputs = {
    # stable = pinned release
    nixpkgs.url = "github:nixos/nixpkgs?channel=24.05";

    # unstable = rolling release
    unstable-nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs, unstable-nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
      unstable-pkgs = import unstable-nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in
    {
      formatter.${system} = pkgs.nixfmt-classic;
      devShells.x86_64-linux = {
        default = pkgs.mkShell rec {
          shellHook = ''
            PS1="[\u \W]λ "
            # this provides libpq, libssl etc defined in the nativeInputs
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${
              pkgs.lib.makeLibraryPath nativeInputs
            }"
            export SOME_ENV_VAR="Hello, World!"

            echo "Welcome to the Webapp Template Shell!"
          '';
          nativeInputs = with pkgs; [
            postgresql.lib
            openssl
            python313
          ];
          buildInputs = with pkgs; [
            act
            awscli2
            cargo-deny
            clang
            curl
            diesel-cli
            docker
            docker-compose
            git
            mold
            nodejs
            pkg-config
            pre-commit
            python313
            rustup
            terraform
          ];
        };
      };
    };
}
