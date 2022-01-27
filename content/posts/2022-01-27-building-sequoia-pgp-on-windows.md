---
title: "Building Sequoia PGP on Windows"
date: 2022-01-27T15:57:21-05:00
draft: false
---

This is more for my own notes, but if you're new to Windows development, it's easy to get lost in the mix of MSVC and 'mingw'. For the purposes of this post, the goal will be to build Sequoia PGP, a rust PGP implementation, in Windows using the 'mingw' environment (which uses the Nettles cryptography library vs Windows Cryptography).

## Summary of the steps

- Install Rustup using the official package
- Install the windows-gnu toolchain and set it to default
- Install MSYS2
- Install the required packages from MSYS2 (clang, nettles, pkg-config)
- Update PATH
- Build using cargo

## Installation steps

### Install rustup

Use the official package at [rustup.rs](https://rustup.rs/)

### Install the Windows GNU toolchain

You could have done this during the installation of rustup, but for clarity, if you went with the default settings, it could be installed separately:

`rustup toolchain install stable-x86_64-pc-windows-gnu`

This default to this toolchain:  
`rustup default stable-x86_64-pc-windows-gnu`

### Install MSYS2

Hop over to [MSYS2.org](https://www.msys2.org/) and follow the instructions there.

### Install the required packages from MSYS2

`pacman -S mingw-w64-x86_64-clang mingw-w64-x86_64-pkg-config mingw-w64-x86_64-nettle`

Alternatively, install 'pacboy' which gets rid of the 'mingw-w64-x86_64' mess:  
`pacman -S pactools`  
`pacboy -S clang pkg-config nettle`

Make sure that you update the path (Assumes you're in Powershell):  
`$env:Path += ';C:\msys64\mingw64\bin'`

This is required because the build process will need to be able to find 'cc.exe' in the path.

## Build Sequoia

Then run `cargo build` and that should be it.

## Resources for this post

- [Stackoverflow about nettles and mingw](https://stackoverflow.com/questions/47379214/step-by-step-instruction-to-install-rust-and-cargo-for-mingw-with-msys2)
- [Sequoia Windows build instructions](https://stackoverflow.com/questions/47379214/step-by-step-instruction-to-install-rust-and-cargo-for-mingw-with-msys2)