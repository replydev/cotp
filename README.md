# cotp - command line totp authenticator
[![Actions Status](https://github.com/replydev/cotp/workflows/Rust/badge.svg)](https://github.com/replydev/cotp/actions)
[![AUR package](https://img.shields.io/aur/version/cotp)](https://aur.archlinux.org/packages/cotp/)
[![crates.io](https://img.shields.io/crates/v/cotp)](https://crates.io/crates/cotp)
[![Downloads](https://img.shields.io/crates/d/cotp)](https://crates.io/crates/cotp)

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

## Overview
### Interface
cotp is written with simplicity in mind, the interface is quite minimalist and intuitive as command line apps should be.
This is an example:

    cotp v0.1.5
    written by @replydev

    Password: 
    [========================                                    ]
    1) Google - mymail@gmail.com: 460984
    2) Github - myusername: 950152
    3) Amazon - myusername: 208235

### Encryption
This program relies on only one database file, encrypted with [XChaCha20Poly1305](https://doc.libsodium.org/advanced/stream_ciphers/xchacha20) authenticated encryption and [Argon2id](https://en.wikipedia.org/wiki/Argon2) for key derivation.
### Import/Export
You can import backups (or [converted databases](#converted-databases)) from:
 - [Aegis](https://github.com/beemdevelopment/Aegis)
 - [andOTP](https://github.com/andOTP/andOTP)
 - [Authy](https://authy.com/)
 - [Google Authenticator](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2)

Backup compatibility is growing (check [planned features](#planned-features)).
By typing `cotp -ex` you can export your database in unencrypted json format.
### Compatibility
cotp can generate two-factor authentication coded using HMAC-SHA1, HMAC-SHA256 and HMAC-SHA512, with any digits, to provide a good compatibility to most two-factor authentication systems.
### Cross Plaform
Thanks to the glorious [Rust Language](https://www.rust-lang.org/) cotp is easily **compilable** in every platform supported by rust itself.
As now i personally tested program functionalities in these systems:
 - Arch Linux
 - Alpine Linux 3.12.3
 - Fedora 33
 - Windows 10 x64 LTSC 1809 Build 17763.1637

## Installation

### ArchLinux
You can install cotp through the Arch User Repository.
Before beginning check you already have the required packages:

`pacman -S git base-devel`

Then choose how you want to proceed:

- Using an AUR Helper like [yay](https://github.com/Jguer/yay): 
`yay -S cotp`
- Or [paru](https://github.com/morganamilo/paru):
`paru -S cotp`
- Manually cloning AUR repo and make the pkg

	```
	git clone https://aur.archlinux.org/cotp.git
	cd cotp
	makepkg -si
	```
### Other distributions, *nix or Windows

Before beginning check that you have the required compilers to build cotp by yourself:
 - gcc for *nix operating systems
 - msvc for Windows
 - rust toolchain, can be installed through [rustup](https://rustup.rs/)
#### Using crates.io repository

It's possible to install cotp directly through cargo, as it's listed in the [crates.io](https://crates.io/crates/cotp) repository.

Just type `cargo install cotp` and wait for the installation.

#### Clone the Github repository and manually install
You can build cotp using these commands:

    git clone https://github.com/replydev/cotp.git #or https://codeberg.org/replydev/cotp.git
    cargo install --path cotp/

## How to use
If you are familiar with the command line interface using cotp will not be a problem.
Please note that cotp requires at least an 8 chars length password.
If you type `cotp -h` you get some instruction on how to use cotp utilities.
For example, the version 0.1.5 prints out this help screen:
```
cotp v0.1.5
written by @replydev

USAGE:
  cotp [SUBCOMMAND]

ARGUMENTS:
  -a,--add [ISSUER] [LABEL] [ALGORITHM] [DIGITS]       | Add a new OTP code
  -e,--edit [ID] [ISSUER] [LABEL] [ALGORITHM] [DIGITS] | Edit an OTP code
  -r,--remove [ID]                                     | Remove an OTP code
  -i,--import [APPNAME] [PATH]                         | Import a backup from a given application
  -ex,--export                                         | Export the entire database in a plaintext json format
  -j,--json                                            | Print results in json format
  -s,--single                                          | Print OTP codes in single mode
  -in,--info [ID]                                      | Print info of choosen OTP code
  -h,--help                                            | Print this help
```
Note that in the `--edit` command if you type . instead of argument you are specifying not to modify that specific argument.
### Example:
#### Before:
|index|issuer|label|algorithm|digits|
|--|--|--|--|--|
| 3 | Email_Provider | mymail@example.com | SHA1 | 6 |
#### Command:

    cotp -e 3 . myothermail@example.com . 8

#### After:
|index|issuer|label|algorithm|digits|
|--|--|--|--|--|
| 3 | Email_Provider | **mymailother@example.com** | SHA1 | **8** |

## Database conversion
To import Authy or Google Authenticator databases you need first of all to obtain the respective files in your phone in the paths: 
- **Authy**: `/data/data/com.authy.authy/shared_prefs/com.authy.storage.tokens.authenticator.xml`
- **Google Authenticator**: `/data/data/com.google.android.apps.authenticator2/databases/databases`

After that run the correct python script located in the converters/ folder in this source code:

`python authy.py path/to/database.xml converted.json`

It will convert the database in a json format readable by cotp.

To finish import the database: `cotp -i authy path/to/database.json`

## Planned features

 - [x] Reduce binary size and improve compilation speed by removing useless dependencies.
 - [x] Use Argon2id for key derivation
 - [x] CLI Dashboard
 - [x] Support for:
   - [x] SHA256
   - [x] SHA512
   - [x] Custom digit value
 - [ ] Backup compatibility with:
	 - [x] Aegis
	 - [x] andOTP
	 - [x] Authy
	 - [x] Google Authenticator
	 - [ ] FreeOTP
 - [ ] Graphical User Interface 

## Contribution
I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I gratefully accept any contributions.
