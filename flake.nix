{
  description = "nextbike_rides_viewer";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      packages = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          viewer = pkgs.buildNpmPackage {
            pname = "viewer";
            version = "0.0.1";
            src = ./viewer;
            npmDepsHash = "sha256-ZdITDfGooOD9md0NE+x/lveSDJoYWM1iLnbDiXAs/Pc=";

            buildPhase = ''
              npm run build
            '';

            installPhase = ''
              mkdir -p $out
              cp -r dist/* $out/
            '';
          };

          backend = pkgs.rustPlatform.buildRustPackage {
            pname = "api";
            version = "0.0.1";
            src = ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [
              self.packages.${system}.viewer
            ];

            postPatch = ''
              # copy frontend to be included in the backend binary
              mkdir -p viewer/dist
              cp -r ${self.packages.${system}.viewer}/index.html viewer/dist/
            '';
          };
        }
      );

      nixosModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.services.nextbike-rides-viewer;
        in
        {
          options.services.nextbike-rides-viewer = {
            enable = lib.mkEnableOption "nextbike rides viewer";

            dataDir = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/nextbike-rides-viewer";
              description = "Directory used for rides.bin and stations.json data files.";
            };
          };

          config = lib.mkIf cfg.enable {
            systemd.services.nextbike-rides-viewer = {
              description = "Nextbike rides viewer";
              wantedBy = [ "multi-user.target" ];
              after = [ "network-online.target" ];
              wants = [ "network-online.target" ];
              serviceConfig = {
                DynamicUser = true;
                StateDirectory = "nextbike-rides-viewer";
                WorkingDirectory = cfg.dataDir;
                ExecStart = "${self.packages.${pkgs.system}.backend}/bin/nextbike";
                Restart = "always";
                RestartSec = 5;
              };
            };
          };
        };
      devShells = forEachSystem (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.rustc
              pkgs.clippy
              pkgs.nodejs
              pkgs.watchexec
              pkgs.mprocs
              pkgs.prek
              pkgs.nixfmt
            ];

            shellHook = ''
              alias up="mprocs 'mkdir viewer/dist; touch viewer/dist/index.html; watchexec --restart --exts rs cargo run' 'cd viewer && npm ci && npm run dev';";
            '';
          };
        }
      );
    };
}
