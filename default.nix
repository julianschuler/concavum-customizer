{pkgs ? import <nixpkgs> {}}: let
  workspaceManifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    meta = with pkgs.lib; {
      description = "Customizer for the Concavum keyboard";
      longDescription = ''
        Interactive customizer for the Concavum, a fully parametric split keyboard featuring an ergonomic layout with ortholinear (non-staggered) columns and concave key wells.
      '';
      license = licenses.gpl3Only;
      homepage = "https://github.com/julianschuler/concavum-customizer";
      mainProgram = "customizer";
    };
    pname = "concavum-customizer";
    version = workspaceManifest.version;
    cargoLock.lockFile = ./Cargo.lock;

    cargoLock.outputHashes = {
      # This hash needs to be updated manually for git dependencies in Cargo.toml unfortunately.
      # After updating the dependency, run `nix build`, let it fail and paste the hash it prints in here.
      "fidget-0.3.4" = "sha256-UhjT6uibly1j79kSFkOzQ6eX3RNPo2eO6gBnHwNHULk=";
    };
    src = pkgs.lib.cleanSource ./.;
    nativeBuildInputs = with pkgs; [
      pkg-config
      makeWrapper
    ];
    buildInputs = with pkgs; [
      libxkbcommon
      libGL

      wayland
      wayland-protocols

      # Not tested on X11, but should work. If you tested it please remove this comment :)
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      xorg.libX11
    ];
    postFixup = ''
      wrapProgram $out/bin/customizer \
        --suffix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath buildInputs}
    '';
  }
