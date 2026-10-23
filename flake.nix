{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/174eb786fb68e3a13e4e535a3deea479a0c07a6a";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      linuxdeployAppImage = pkgs.runCommandLocal "linuxdeploy" {
        version = "1-alpha-20251107-1";
        src = pkgs.fetchurl {
          url = "https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage";
          hash = "sha256-wgzXHjpOO4DDSDzveTzaP06ZCsoUAU0jxUTKPOEnC00=";
        };
        nativeBuildInputs = [ pkgs.squashfs-tools ];
      } ''
        mkdir -p $out/bin $out/libexec
        squashfs_offset=$(grep -abo 'hsqs' $src | awk -F: 'END { print $1 }')
        unsquashfs -o "$squashfs_offset" -d $out/libexec/linuxdeploy $src >/dev/null
        sed -i '1s|^#! */bin/bash|#!/bin/sh|' $out/libexec/linuxdeploy/plugins/linuxdeploy-plugin-appimage/usr/bin/appimagetool

        linuxdeploy_dir="$out/libexec/linuxdeploy"
        cat > $out/bin/linuxdeploy <<WRAPPER
        #!/bin/sh
        export APPDIR="$linuxdeploy_dir"
        exec "$linuxdeploy_dir/AppRun" "\$@"
        WRAPPER
        chmod +x $out/bin/linuxdeploy
      '';
    in
    {
      packages.${system} = {
        inherit linuxdeployAppImage;
        default = linuxdeployAppImage;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          nodejs_26
          cargo
          clippy
          desktop-file-utils
          pkg-config
          gobject-introspection
          at-spi2-atk
          atkmm
          binutils
          cairo
          file
          gdk-pixbuf
          glib
          gtk3
          harfbuzz
          librsvg
          libsoup_3
          libxcb
          libxkbcommon
          linuxdeployAppImage
          mesa
          openssl
          pango
          patchelf
          rustc
          rustfmt
          squashfs-tools
          wayland
          webkitgtk_4_1
          xdg-utils
          xkeyboard-config
          glib-networking
          dconf

          # alsa-lib is for Everyone Voice Chat to work in dev mode.
          alsa-lib
        ];

        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
          export GIO_EXTRA_MODULES="${pkgs.dconf.lib}/lib/gio/modules:${pkgs.glib-networking}/lib/gio/modules"
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [ pkgs.alsa-lib pkgs.libGL pkgs.libxkbcommon pkgs.mesa pkgs.wayland ]}$LD_LIBRARY_PATH"
          export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
          export GBM_BACKENDS_PATH="${pkgs.mesa}/lib/gbm"
          export __EGL_VENDOR_LIBRARY_FILENAMES="${pkgs.mesa}/share/glvnd/egl_vendor.d/50_mesa.json"
        '';
      };
    };
}
