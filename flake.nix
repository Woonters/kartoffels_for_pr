{
  inputs = {
    crane = {
      url = "github:ipetkov/crane";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    napalm = {
      url = "github:nix-community/napalm";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-24.05";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";

      inputs = {
        nixpkgs = {
          follows = "nixpkgs";
        };
      };
    };

    utils = {
      url = "github:numtide/flake-utils";
    };
  };

  outputs = { self, crane, napalm, nixpkgs, rust-overlay, utils }:
    let
      packages = utils.lib.eachDefaultSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [
              (import rust-overlay)
            ];
          };

          backend = import ./backend {
            inherit crane pkgs;
          };

          frontend = import ./frontend {
            inherit napalm pkgs;
          };

        in
        {
          packages = {
            inherit backend frontend;

            default = pkgs.linkFarm "kartoffels" [
              { name = "backend"; path = backend; }
              { name = "frontend"; path = frontend; }
            ];
          };

          devShells = {
            default = pkgs.mkShell {
              buildInputs = with pkgs; [
                just
                nodejs
              ];
            };
          };
        }
      );

      others = {
        nixosConfigurations = {
          container = import ./nixos/container.nix {
            inherit self nixpkgs;
          };
        };

        nixosModules = {
          default = import ./nixos/module.nix {
            inherit self;
          };
        };
      };

    in
    packages // others;
}
