# Install cotp 

## Arch Linux and arch-based distributions
We have two AUR packages ready to be installed, [cotp](https://aur.archlinux.org/packages/cotp) and [cotp-bin](https://aur.archlinux.org/packages/cotp-bin). You can use an AUR Helper like [paru](https://github.com/morganamilo/paru):

`paru -S cotp`

Or manually clone the AUR repo and make the pkg
```
pacman -S git base-devel
git clone https://aur.archlinux.org/cotp.git
cd cotp
makepkg -si
```

## Windows

Windows installation/compilation is supported with both of toolchains.

If you want to use `x86_64-pc-windows-msvc` you will need to install the [Visual C++ 2019 Build Tools](https://visualstudio.microsoft.com/it/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)

Once you have the rust toolchain installed just run `cargo install cotp`.
  

## Other linux distributions and \*nix
Before beginning check that you have the required build dependencies to use the rust compiler.

You need to install the **libxcb-devel** dependency, needed for clipboard coping in X11:

- Debian based: `sudo apt install libxcb1-dev`
- RHEL based: `sudo dnf install libxcb-devel`
- Arch based: `sudo pacman -S lib32-libxcb`

### Use the crates.io repository

Just type `cargo install cotp` and wait for the installation.

### Clone the GitHub repository and manually install

You can build cotp using these commands:
```
git clone https://github.com/replydev/cotp.git
cargo install --path cotp/
```
