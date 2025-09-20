{
  description = "tat development flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }: let 
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      buildInputs = with pkgs; [
        cargo rustc rust-analyzer gdal
      ];

      nativeBuildInputs = [ pkgs.pkg-config ];
      env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
      env.PKG_CONFIG_PATH = "${pkgs.gdal}/lib/pkgconfig";
      env.LIBCLANG_PATH = "${pkgs.llvmPackages_21.libclang.lib}/lib";
    };
  };
}
