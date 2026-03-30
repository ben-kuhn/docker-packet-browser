{
  description = "Packet Browser - Secure web browser for packet radio";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default;

        packet-browser = pkgs.rustPlatform.buildRustPackage {
          pname = "packet-browser";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.rustfmt  # Required by headless_chrome build
          ];
          buildInputs = [ pkgs.openssl ];

          # Skip tests in Nix build - they require single-threaded execution
          # due to env var manipulation. Tests are run separately in CI.
          doCheck = false;
        };

        dockerImage = pkgs.dockerTools.buildImage {
          name = "docker-packet-browser";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              packet-browser
              pkgs.chromium
              pkgs.dumb-init
              pkgs.logrotate
              pkgs.cacert
              pkgs.fontconfig       # Runtime font configuration (fontconfig.conf, fc-cache)
              pkgs.liberation_ttf   # Core font set (replaces Arial/Times/Courier)
              pkgs.noto-fonts       # Wide Unicode coverage for packet radio text pages
            ];
            pathsToLink = [ "/bin" "/etc" "/share" ];
          };

          config = {
            Cmd = [ "/bin/dumb-init" "/bin/packet-browser" ];
            ExposedPorts = { "63004/tcp" = {}; };
            Env = [
              "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
              # buildEnv does not create /etc/fonts at the root level; point fontconfig
              # directly at the store path so Chrome's font manager initialises correctly.
              "FONTCONFIG_FILE=${pkgs.fontconfig.out}/etc/fonts/fonts.conf"
            ];
            User = "1000:1000";
          };

          runAsRoot = ''
            mkdir -p /var/log/packet-browser
            mkdir -p /tmp
            chown 1000:1000 /var/log/packet-browser
          '';
        };
      in
      {
        packages = {
          default = packet-browser;
          docker-image = dockerImage;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.openssl
            pkgs.chromium
          ];
        };
      }
    );
}
