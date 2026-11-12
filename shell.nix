{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [
    pkg-config
    cargo
    rustc
  ];

  buildInputs = [
    # --- X11 / windowing (winit 0.30 dlopen'ит libX11.so.6) ---
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXext
    libxkbcommon          # winit 0.30 требует libxkbcommon + xkbcommon-x11
    xorg.xkeyboardconfig

    # --- Аудио / ввод / GPU ---
    alsa-lib
    udev
    vulkan-loader
    vulkan-headers
    vulkan-tools          # `vulkaninfo` для диагностики, если что-то с GPU

    # --- Mesa: программный/аппаратный Vulkan (Intel/AMD) ---
    mesa
  ];

  # Ключевая строка: makeLibraryPath собирает все /nix/store/.../lib в один
  # LD_LIBRARY_PATH, который видят и линкер, и dlopen. Без неё winit падает с
  # "cannot open shared object file: libX11.so.6".
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

  shellHook = ''
    echo "rounds_white dev shell — $(rustc --version)"
    echo "LD_LIBRARY_PATH set, X11 + Vulkan ready. Run: cargo run"
  '';
}