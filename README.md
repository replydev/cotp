# cotp - command line TOTP/HOTP authenticator

[![Actions Status](https://github.com/replydev/cotp/workflows/Build/badge.svg)](https://github.com/replydev/cotp/actions) [![AUR package](https://img.shields.io/aur/version/cotp)](https://aur.archlinux.org/packages/cotp/) [![crates.io](https://img.shields.io/crates/v/cotp)](https://crates.io/crates/cotp) [![Downloads](https://img.shields.io/crates/d/cotp)](https://crates.io/crates/cotp)

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

# Overview

## Interface

cotp is written with simplicity in mind, the interface is quite minimalist and intuitive as command line apps should be.

[![asciicast](https://asciinema.org/a/459912.svg)](https://asciinema.org/a/459912)

If you are familiar with the command line interface using cotp will not be a problem. Just type `cotp` to enter the TUI dashboard.
In the first run you will be prompted to insert a password to initialize the database.
Please note that the software requires at least an 8 chars length password.

If you type `cotp --help` you get some instruction on how to use cotp utilities.
The interface is divided in subcommands, so if you type `cotp <subcommand> --help` you get options and flag relative to the subcommand you inserted.

## Encryption

This program relies on only one database file encrypted with [XChaCha20Poly1305](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) authenticated encryption and [Argon2id](https://en.wikipedia.org/wiki/Argon2) for key derivation.
It also uses [AES-GCM](https://docs.rs/aes-gcm/latest/aes_gcm/) to import from encrypted Aegis backups.

## Import/Export

You can import backups (or [converted databases](#database-conversion)) from:

-  [Aegis](https://github.com/beemdevelopment/Aegis) (Both encrypted and plain database types)
-  [andOTP](https://github.com/andOTP/andOTP)
-  [FreeOTP](https://github.com/freeotp/freeotp-android)
-  [FreeOTP+](https://github.com/helloworld1/FreeOTPPlus)
-  [Authy](https://authy.com/)
-  [Google Authenticator](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2)
-  [Microsoft Authenticator](https://play.google.com/store/apps/details?id=com.azure.authenticator)

Backup compatibility is growing (check [planned features](#planned-features)).
By typing `cotp export` you can export your database in unencrypted json format.

## Compatibility

cotp can generate both **TOTP** and **HOTP** codes, compliant with **rfc6238** and **rfc4226** specifications. Also, it is possible to customize settings like **HMAC algorithm** and **digits**, to provide a good compatibility to other two-factor authentication systems.

### Cross Plaform

#### So far, I have successfully tested the functionality of the software in the following systems:

-  Arch Linux
-  Alpine Linux
-  Fedora 33
-  Ubuntu 20.04 WSL
-  Windows 10 Pro
-  Windows 10 LTSC
-  Windows 11

#### In addition, cotp has been successfully tested by the community in the following systems:

-  NixOS

## Copy to clipboard

You can copy the otp code of the element you selected by simply pressing enter.
This is supported in Windows, macOS, X11 and Wayland.

## Increment HOTP counter

To increment or decrement the HOTP counter just press '+' or '-'

## Add OTP Code

Just type `cotp add -i <issuer>`, press Enter and insert the BASE32 Secret Key.
cotp also support HOTP codes, just add the `--hotp` flag the the `--digits` value.
Type `cotp add --help` to learn how to insert other settings.

## Edit OTP Code

You can edit your codes with the edit subcommand.

You must indicate the index of the code to be edited with the **--index** argument and then indicate the fields to be edited.
If you want to modify also the secret of the code you must insert the flag **-c**.

# Install

See [install.md](install.md)

# Migration from other apps

See [codes_migration.md](codes_migration.md)

## Planned features

-  [x] Reduce binary size and improve compilation speed by removing useless dependencies.
-  [x] Use Argon2id for key derivation
-  [x] CLI Dashboard
-  [x] Support for:
   -  [x] SHA256
   -  [x] SHA512
   -  [x] Custom digit value
-  [x] Backup compatibility with:
   -  [x] Aegis
   -  [x] andOTP
   -  [x] Authy
   -  [x] Google Authenticator
   -  [x] Microsoft Authenticator
   -  [x] FreeOTP
   -  [x] FreeOTP+

## Contribution

I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I gratefully accept any contributions.
