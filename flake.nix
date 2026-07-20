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

      # Fast, hermetic hygiene checks: Nix formatting/lint, markdown, and
      # whitespace. This is deon's founding toolchain leg — it mirrors Pacioli's
      # hygiene set *minus* the Lean/nix-proof gates (deon has no Lean yet). The
      # Rust `deon-check` binary and its fmt/clippy/test gate join here when the
      # checker lands; the Lean seam later still.
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
      checks = forAllSystems (system: {
        pre-commit = hooksFor system;
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
            buildInputs = hooks.enabledPackages;
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
