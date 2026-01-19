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
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        ffmpeg
        fmt.dev
        cudaPackages.cuda_cudart
        cudatoolkit
        nvidiaPackage
        cudaPackages.cudnn
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
        export CUDA_PATH=${pkgs.cudatoolkit}
        export EXTRA_LDFLAGS="-L/lib -L${nvidiaPackage}/lib"
        export EXTRA_CCFLAGS="-I/usr/include"
        export CMAKE_PREFIX_PATH="${pkgs.fmt.dev}:$CMAKE_PREFIX_PATH"
        export PKG_CONFIG_PATH="${pkgs.fmt.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
      '';
    };
  };
}