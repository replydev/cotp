# cotp - command line totp authenticator

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

## Overview
cotp is written with simplicity in mind, it relies on only one database file, encrypted with XChaCha20Poly1305 authenticated encryption and scryptsalsa208sha256 for key derivation. Use of argon2id13 KDF is planned.

The interface is quite minimalist and intuitive, by typing `cotp -h` you can see all the program features.

You can also import backup made by [Aegis](https://github.com/beemdevelopment/Aegis) and [andOTP](https://github.com/andOTP/andOTP) Authenticators, but backup compatibility is growing (check [planned features](##planned-features))

## Installation

### ArchLinux
You can install cotp through the Arch User Repository.
Before beginning check you already have the required packages:

`pacman -S git base-devel`

Then choose how you want to proceed

- Using an AUR Helper like [yay]("https://github.com/Jguer/yay"): 
`yay -S cotp`
- Manually cloning AUR repo and make the pkg

	```
	git clone https://aur.archlinux.org/cotp.git
	cd cotp
	makepkg -si
	```
### Other distributions
You can use prebuilt binaries or [build cotp](##building) yourself
Go to [releases page](https://github.com/replydev/cotp/releases/) and get the latest version.


## Building
First of all install the rust toolchain:

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Now clone the repository and build the release binary:

    git clone https://github.com/replydev/cotp.git #or https://codeberg.org/replydev/cotp.git
    cd cotp
    cargo build --release
You will find the compiled binary in **target/release** folder

## Planned features

 - Reduce binary size and improve compilation speed by removing useless dependencies.
 - Use argon2id13 for key derivation
 - Backup compatibility with:
	 - [x] Aegis
	 - [x] andOTP
	 - [ ] Authy
	 - [ ] Google Authenticator
	 - [ ] FreeOTP
 - Graphical User Interface 

## Contribution
I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I gratefully accept any contributions.
  

