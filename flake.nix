{
  description = "CUDA development environment";
  outputs = {
    self,
    nixpkgs,
  }: let
    system = "aarch64-linux";
    pkgs = import nixpkgs {
      inherit system;
      config.allowUnfree = true;
      config.cudaSupport = true;
      config.cudaVersion = "13";
    };
    # Change according to the driver used: stable, beta
    nvidiaPackage = pkgs.linuxPackages.nvidiaPackages.beta;
  in {
    devShells.${system}.default = pkgs.mkShell rec  {
      buildInputs = with pkgs; [
        gcc
        cudaPackages_13.cuda_cudart
        cudaPackages_13.cudatoolkit
        nvidiaPackage
        cudaPackages_13.cudnn
        libGL
        xorg.libXi
        xorg.libXmu
        xorg.libXext
        xorg.libX11
        xorg.libXv
        xorg.libXrandr
      ];

      LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      LD_LIBRARY_PATH = "${nvidiaPackage}/lib:$LD_LIBRARY_PATH";
      CUDA_PATH = "${pkgs.cudaPackages_13.cudatoolkit}";
      shellHook = ''
        export BINDGEN_EXTRA_CLANG_ARGS="$(< ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/libc-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/cc-cflags) \
          $(< ${pkgs.stdenv.cc}/nix-support/libcxx-cxxflags) \
          ${pkgs.lib.optionalString pkgs.stdenv.cc.isClang "-idirafter ${pkgs.stdenv.cc.cc}/lib/clang/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"} \
          ${pkgs.lib.optionalString pkgs.stdenv.cc.isGNU "-isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc} -isystem ${pkgs.stdenv.cc.cc}/include/c++/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/${pkgs.stdenv.hostPlatform.config}"}
        "
      '';      
    };
  };
}