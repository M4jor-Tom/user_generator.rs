{
  description = "Random user profile generator with clipboard support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;

      # Vendor dependencies using `cargo vendor` instead of nix's built-in
      # Python fetcher, which gets blocked by crates.io for lacking a proper
      # User-Agent. This is a fixed-output derivation (has network access).
      cargoVendorDir = pkgs.stdenv.mkDerivation {
        name = "user_generator-vendor";
        src = lib.cleanSource ./.;
        nativeBuildInputs = with pkgs; [ cargo rustc cacert ];
        outputHashMode = "recursive";
        outputHashAlgo = "sha256";
        outputHash = "sha256-TKD5LANLJM2HTavJl/DYbXjcClQMO3BMQpp2ittOjHo=";
        phases = [ "unpackPhase" "buildPhase" ];
        buildPhase = ''
          sourceRoot="$(ls -d */)"
          cd "$sourceRoot"
          export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
          HOME=$TMPDIR cargo vendor --locked "$out"
        '';
        installPhase = "true";
      };

      # Base Rust package
      # Uses cargo build --release --frozen with pre-vendored deps instead of
      # buildRustPackage, because crates.io blocks nix's Python crate fetcher.
      rustPackage = pkgs.stdenv.mkDerivation {
        pname = "user_generator";
        version = "0.1.0";
        src = lib.cleanSource ./.;
        nativeBuildInputs = with pkgs; [ cargo rustc ];

        buildPhase = ''
          vendorDir=${cargoVendorDir}
          cp -Lr --reflink=auto "$vendorDir" vendor
          chmod -R +w vendor

          mkdir -p .cargo
          cat > .cargo/config.toml <<EOF
          [source.crates-io]
          replace-with = "vendored-sources"

          [source.vendored-sources]
          directory = "$(pwd)/vendor"
          EOF

          cargo build --release --frozen
        '';

        installPhase = ''
          install -Dm755 target/release/user_generator -t $out/bin
        '';
      };

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
          name = "user_generator";
          fields = "email,password,username,fullname";
          passwordMinLength = 12;
          passwordRequireUpper = true;
          passwordRequireSpecial = true;
          passwordRequireDigit = true;
        };
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
        ];
      };
    };
}
