{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    llvmPackages_20.libllvm
    llvmPackages_20.llvm
    llvmPackages_20.libclang

    pkg-config
    clang
    zlib
    libxml2

    rust-bindgen
    #bpf-linker
  ];

  # Optionally, set environment variables if bindgen needs to find libclang
  # Remove this if not necessary in your case
  shellHook = ''
    export LIBCLANG_PATH=${pkgs.lib.makeLibraryPath [pkgs.llvmPackages_20.libclang.lib]}
    export LLVM_CONFIG_PATH=${pkgs.llvmPackages_20.llvm}/bin/llvm-config
  '';
}
