{
  description = "deon — colored deontic norm language + static checker (judgment-side sibling to Pacioli)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { nixpkgs, git-hooks, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      # The Rust `deon-check` binary (the leak-detection static check, DESIGN §4
      # check 1) built hermetically: `cargo fmt --check` + `cargo clippy
      # -D warnings` gate the build, and `cargo test` (buildRustPackage's default
      # checkPhase) runs the acceptance suite (seed norms clean; leaky fixture →
      # 3 located leaks). Deps are vendored from Cargo.lock, so it needs no
      # network. Exposed as both `packages.default` and a flake check, so
      # `nix flake check` — the one required CI status — covers Rust too.
      deonCheckFor =
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "deon-check";
          version = "0.1.0";
          # Only the crate inputs — keeps the build pure and off target/ etc.
          # examples/ and tests/ are included: the acceptance tests read them.
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./src
              ./tests
              ./examples
            ];
          };
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = [
            pkgs.clippy
            pkgs.rustfmt
          ];
          preBuild = ''
            cargo fmt --check
            cargo clippy --all-targets -- -D warnings
          '';
        };

      # Fast, hermetic hygiene checks: Nix formatting/lint, markdown, and
      # whitespace. Mirrors Pacioli's hygiene set *minus* the Lean/nix-proof
      # gates (deon has no Lean yet); the Lean seam joins later.
      hooksFor =
        system:
        git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # `nixfmt` (not `nixfmt-rfc-style`): as of nixpkgs 25.11 the RFC 166
            # formatter *is* `pkgs.nixfmt`, and the old alias warns on eval.
            nixfmt.enable = true;
            deadnix.enable = true;
            statix.enable = true;
            check-merge-conflicts.enable = true;
            check-added-large-files.enable = true;
            trim-trailing-whitespace.enable = true;
            end-of-file-fixer.enable = true;
            check-yaml.enable = true;
            markdownlint = {
              enable = true;
              settings.configuration = {
                MD013 = {
                  # line length — prose wraps at 80 for terminal review; tables
                  # and code blocks (the abstract-syntax grammar, ASCII) can't
                  # reflow, so exempt them.
                  line_length = 80;
                  tables = false;
                  code_blocks = false;
                };
                MD033 = false; # inline HTML
                MD036 = false; # emphasis-as-heading — prose uses emphasis stylistically
                MD040 = false; # fenced code language not required (grammar blocks)
                MD025.front_matter_title = ""; # OKF norm files carry a YAML front-matter title
              };
            };
          };
        };
    in
    {
      packages = forAllSystems (system: {
        default = deonCheckFor system;
      });

      checks = forAllSystems (system: {
        pre-commit = hooksFor system;
        deon-check = deonCheckFor system;
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          hooks = hooksFor system;
        in
        {
          default = pkgs.mkShell {
            inherit (hooks) shellHook;
            # hygiene tools + the Rust toolchain for local `cargo` work (mkShell's
            # stdenv provides the C compiler the build scripts link against).
            buildInputs = hooks.enabledPackages ++ [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.rustfmt
            ];
          };
        }
      );

      formatter = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        pkgs.writeShellApplication {
          name = "fmt";
          runtimeInputs = [
            pkgs.nixfmt
            pkgs.findutils
          ];
          text = ''
            find . -name '*.nix' -not -path './.git/*' -print0 | xargs -0 nixfmt
          '';
        }
      );
    };
}
