{
  description = "Random user profile generator with clipboard support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      craneLib = crane.mkLib pkgs;

      commonArgs = {
        src = craneLib.cleanCargoSource ./.;
      };

      cargoArtifacts = craneLib.buildDepsOnly commonArgs;

      rustPackage = craneLib.buildPackage (commonArgs // {
        inherit cargoArtifacts;
      });

      # Helper to create a profile wrapper that sets env vars
      buildProfile = {
        name ? "user_generator",
        fields ? "email,password,first,last",
        passwordMinLength ? "8",
        passwordRequireUpper ? false,
        passwordRequireSpecial ? false,
        passwordRequireDigit ? false,
      }:
        let
          inherit (pkgs) writeShellApplication;
        in
        writeShellApplication {
          inherit name;
          runtimeInputs = [ rustPackage pkgs.wl-clipboard ];
          text = ''
            export FIELDS="${fields}"
            export PASSWORD_MIN_LENGTH="${toString passwordMinLength}"
            export PASSWORD_REQUIRE_UPPER="${if passwordRequireUpper then "true" else "false"}"
            export PASSWORD_REQUIRE_SPECIAL="${if passwordRequireSpecial then "true" else "false"}"
            export PASSWORD_REQUIRE_DIGIT="${if passwordRequireDigit then "true" else "false"}"
            exec user_generator "$@"
          '';
        };

    in {
      packages.${system} = {
        # Default: just the binary, no profile restrictions
        default = buildProfile {
          name = "user_generator";
        };

        # Hugging Face profile: stricter password requirements
        huggingface = buildProfile {
          # distinct binary name so it can be installed alongside `default`
          name = "huggingface";
          fields = "email,password,username,fullname";
          passwordMinLength = 12;
          passwordRequireUpper = true;
          passwordRequireSpecial = true;
          passwordRequireDigit = true;
        };
      };

      # every profile is runnable: nix run .#huggingface
      apps.${system} = builtins.mapAttrs
        (n: p: {
          type = "app";
          program = pkgs.lib.getExe p;
          meta.description = "user_generator, ${n} profile";
        })
        self.packages.${system};

      devShells.${system}.default = craneLib.devShell {
        packages = with pkgs; [ rust-analyzer ];
      };
    };
}
