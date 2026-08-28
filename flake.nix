{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/174eb786fb68e3a13e4e535a3deea479a0c07a6a";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
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
          cairo
          gdk-pixbuf
          glib
          gtk3
          harfbuzz
          librsvg
          libsoup_3
          openssl
          pango
          rustc
          rustfmt
          webkitgtk_4_1
          xdg-utils
        ];
      };
    };
}
