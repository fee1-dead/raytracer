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
    # Make find-cuda-helper happy on nix
    lib64Shim = pkgs.runCommand "lib64-shim" {} ''
      mkdir -p $out
      ln -s ${pkgs.cudaPackages_13.cudatoolkit}/lib $out/lib64
    '';
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        ffmpeg
        fmt.dev
        cudaPackages_13.cuda_cudart
        cudaPackages_13.cudatoolkit
        nvidiaPackage
        cudaPackages_13.cudnn
        libGLU
        libGL
        xorg.libXi
        xorg.libXmu
        freeglut
        xorg.libXext
        xorg.libX11
        xorg.libXv
        xorg.libXrandr
        zlib
        ncurses
        stdenv.cc
        binutils
        uv
      ];

      shellHook = ''
        export LD_LIBRARY_PATH="${nvidiaPackage}/lib:$LD_LIBRARY_PATH"
        export CUDA_PATH=${pkgs.cudaPackages_13.cudatoolkit}
        export CUDA_LIBRARY_PATH=${lib64Shim}
        export EXTRA_LDFLAGS="-L/lib -L${nvidiaPackage}/lib"
        export EXTRA_CCFLAGS="-I/usr/include"
        export CMAKE_PREFIX_PATH="${pkgs.fmt.dev}:$CMAKE_PREFIX_PATH"
        export PKG_CONFIG_PATH="${pkgs.fmt.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
      '';
    };
  };
}