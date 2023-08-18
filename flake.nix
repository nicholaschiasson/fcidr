{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShell = pkgs.mkShell {
        buildInputs = [
          pkgs.cargo-watch
          pkgs.iconv
          pkgs.just
          pkgs.nerdfonts
          pkgs.rustup
          pkgs.starship
        ];
        shellHook = ''
          source .env
          rustup default stable
          eval "$(starship init bash)"
        '';
      };
    });
}
