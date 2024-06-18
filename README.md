# cotp - command line TOTP/HOTP authenticator

[![Actions Status](https://github.com/replydev/cotp/workflows/Build/badge.svg)](https://github.com/replydev/cotp/actions)
[![crates.io](https://img.shields.io/crates/v/cotp)](https://crates.io/crates/cotp) 
[![Downloads](https://img.shields.io/crates/d/cotp)](https://crates.io/crates/cotp)

[![Packaging status](https://repology.org/badge/vertical-allrepos/cotp.svg)](https://repology.org/project/cotp/versions)

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a
minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

# Overview

## Interface

cotp is written with simplicity in mind, the interface is quite minimalist and intuitive as command line apps should be.

[![asciicast](https://asciinema.org/a/459912.svg)](https://asciinema.org/a/459912)

If you are familiar with the command line interface using cotp will not be a problem. Just type `cotp` to enter the TUI
dashboard. Type `i` to get some instruction. Otherwise just enter `cotp --help`.

In the first run you will be prompted to insert a password to initialize the database.

## Basic functionalities

### Display all the OTP codes in the interactivee dashboard

```
cotp
```

### Add a new TOTP code from a BASE32 secret key

```
cotp add -l <label> -i <optional_issuer>
Password: <insert your database password>
Insert the secret: <BASE32 secret>
```

### Add a new HOTP code with custom algorithm and digits

```
cotp add --type hotp --algorithm SHA256 -d 8 --counter 10
```

BASE32 secret will be prompted as usual

## Encryption

This program relies on only one database file encrypted
with [XChaCha20Poly1305](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) authenticated encryption
and [Argon2id](https://en.wikipedia.org/wiki/Argon2) for key derivation.

It also uses [AES-GCM](https://docs.rs/aes-gcm/latest/aes_gcm/) to import from encrypted Aegis backups.

## Compatibility

cotp can generate both **TOTP** and **HOTP** codes, compliant with **rfc6238** and **rfc4226** specifications. Also, it
is possible to customize settings like **HMAC algorithm** and **digits**, to provide compatibility to other two-factor
authentication systems.

Latest releases also include support for Steam, Yandex, MOTP codes and code copying from SSH Remote Shell.

## Cross Plaform

cotp should be easily compiled on the most used platform, but it is mostly tested on Linux and Windows.

# Install

## Arch Linux and arch-based distributions

Arch Linux has an [official package](https://archlinux.org/packages/extra/x86_64/cotp) in the [extra] repository:

```
pacman -S cotp
```

Additionally if you wish to compile and run the Git HEAD version instead of the current stable release,
an AUR package [cotp-git](https://aur.archlinux.org/packages/cotp-git) can be installed using the
[usual instructions](https://wiki.archlinux.org/title/Arch_User_Repository#Installing_and_upgrading_packages)
or your favorite AUR helper like [paru](https://github.com/morganamilo/paru) (`paru -S cotp-git`).

## NixOS

Check the [official package](https://search.nixos.org/packages?channel=23.11&from=0&size=50&sort=relevance&type=packages&query=cotp).

## Other linux distributions and \*nix

Before beginning check that you have the required build dependencies to use the rust compiler.

You need to install the **libxcb-devel** dependency, needed for clipboard coping in X11:

- Debian based: `sudo apt install libxcb1-dev libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev`
- Fedora / RHEL based: `sudo dnf install libX11-devel`
- Void Linux `sudo xbps-install -S libxcb-devel`

## macOS

```
brew install cotp
```

## Windows

Building is supported with both `x86_64-pc-windows-gnu` and `x86_64-pc-windows-msvc` toolchains.

If you want to use `x86_64-pc-windows-msvc` you will need to install
the [Visual C++ 2019 Build Tools](https://visualstudio.microsoft.com/it/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)

Once you have the rust toolchain installed just run `cargo install cotp`.

### Use the crates.io repository

Just type `cargo install cotp` and wait for the installation.

### Clone the GitHub repository and manually install

You can build cotp using these commands:

```
git clone https://github.com/replydev/cotp.git
cargo install --path cotp/
```

# Migration from other apps

cotp supports TOTP codes migration from various apps.
Some needs to be converted using simple python script you can find listed in the table below.

| App                                                                                                          | How to fetch backup                                                                                                                                                 | Needs conversion                                                          | cotp argument               |
|--------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------|-----------------------------|
| [andOTP](https://github.com/andOTP/andOTP)                                                                   | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--andotp`                  |
| [Aegis](https://github.com/beemdevelopment/Aegis)                                                            | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--aegis`                   |
| [Aegis](https://github.com/beemdevelopment/Aegis) (encrypted)                                                | Make an encrypted backup using the app itself.                                                                                                                      | No                                                                        | `--aegis-encrypted`         |
| [Authy](https://authy.com/)                                                                                  | Obtain `/data/data/com.authy.authy/shared_prefs/com.authy.storage.tokens.authenticator.xml` from your phone.                                                        | [Yes](https://github.com/replydev/cotp/blob/master/converters/authy.py)   | `--authy`                   |
| [Authy](https://authy.com/) (2nd method)                                                                     | Follow this guide: https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93.                                                                              | No                                                                        | `--authy-exported`          |
| [cotp](https://github.com/replydev/cotp)                                                                     | Export your database using `cotp export`.                                                                                                                           | No                                                                        | `--cotp`                    |
| [FreeOTP](https://freeotp.github.io/)                                                                        | Obtain `/data/data/org.fedorahosted.freeotp/shared_prefs/tokens.xml` from your phone.                                                                               | [Yes](https://github.com/replydev/cotp/blob/master/converters/freeotp.py) | `--freeotp`                 |
| [FreeOTP+](https://github.com/helloworld1/FreeOTPPlus)                                                       | Make a backup using the app itself.                                                                                                                                 | No                                                                        | `--freeotp-plus`            |
| [Google Authenticator](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2) | Obtain `/data/data/com.google.android.apps.authenticator2/databases/databases` from your phone                                                                      | [Yes](https://github.com/replydev/cotp/blob/master/converters/gauth.py)   | `--google-authenticator`    |
| [Microsoft Authenticator](https://play.google.com/store/apps/details?id=com.azure.authenticator)             | Obtain `/data/data/com.azure.authenticator/databases/PhoneFactor` from your phone. Take also `PhoneFactor-wal`, `PhoneFactor-shm` if they exist in the same folder. | [Yes](https://github.com/replydev/cotp/blob/master/converters/mauth.py)   | `--microsoft-authenticator` |
| [OTP URI list](https://docs.yubico.com/yesdk/users-manual/application-oath/uri-string-format.html)           | Create a JSON file which contains a items property. It will contains a string array where each element is an OTP URI.                                               | No                                                                        | `--otp-uri`                 |

## How to convert

Once you got the correct files run the right python script located in the **converters/** folder in this source code.

**Example:**
`python authy.py path/to/database.xml converted.json`

It will convert the database in a json format readable by cotp.

To terminate the import:
`cotp import --authy --path path/to/converted_database.json`

# Configuration

By default database is located in `$HOME/.cotp/db.cotp`. If you want to use a different path, you can use `COTP_DB_PATH` environment variable. 
Here is an example of how to do this in bash:
```bash
$ COTP_DB_PATH=/home/user/.local/custom-folder/db.cotp /usr/bin/cotp
```

# Planned features

Currently, there is not any planned feature. If you need something new that could improve the software feel free to open
an issue.

# Contribution

I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I
gratefully accept any pull requests.
