{
  description = "Packet Browser - Secure web browser for packet radio";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
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
          name = "packet-browser";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [
              packet-browser
              pkgs.chromium
              pkgs.dumb-init
              pkgs.logrotate
              pkgs.cacert
            ];
            pathsToLink = [ "/bin" "/etc" ];
          };

          config = {
            Cmd = [ "/bin/dumb-init" "/bin/packet-browser" ];
            ExposedPorts = { "63004/tcp" = {}; };
            Env = [
              "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
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
