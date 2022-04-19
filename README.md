# cotp - command line TOTP/HOTP authenticator

[![Actions Status](https://github.com/replydev/cotp/workflows/Build/badge.svg)](https://github.com/replydev/cotp/actions) [![AUR package](https://img.shields.io/aur/version/cotp)](https://aur.archlinux.org/packages/cotp/) [![crates.io](https://img.shields.io/crates/v/cotp)](https://crates.io/crates/cotp) [![Downloads](https://img.shields.io/crates/d/cotp)](https://crates.io/crates/cotp)

I believe that security is of paramount importance, especially in this digital world. I created cotp because I needed a minimalist, secure, desktop accessible software to manage my two-factor authentication codes.

# Overview

## Interface

cotp is written with simplicity in mind, the interface is quite minimalist and intuitive as command line apps should be.

[![asciicast](https://asciinema.org/a/459912.svg)](https://asciinema.org/a/459912)

  
If you are familiar with the command line interface using cotp will not be a problem. Just type `cotp` to enter the TUI dashboard. Type `i` to get some instruction. Otherwise just enter `cotp --help`.

In the first run you will be prompted to insert a password to initialize the database.

## Encryption

This program relies on only one database file encrypted with [XChaCha20Poly1305](https://docs.rs/chacha20poly1305/latest/chacha20poly1305/) authenticated encryption and [Argon2id](https://en.wikipedia.org/wiki/Argon2) for key derivation.

It also uses [AES-GCM](https://docs.rs/aes-gcm/latest/aes_gcm/) to import from encrypted Aegis backups.

## Compatibility

cotp can generate both **TOTP** and **HOTP** codes, compliant with **rfc6238** and **rfc4226** specifications. Also, it is possible to customize settings like **HMAC algorithm** and **digits**, to provide compatibility to other two-factor authentication systems.

### Cross Plaform

#### So far, I have successfully tested the functionality of the software in the following systems:

- Arch Linux
- Alpine Linux
- Fedora 33
- Ubuntu 20.04 WSL
- Windows 10 Pro
- Windows 10 LTSC
- Windows 11

#### In addition, cotp has been successfully tested by the community in the following systems:

- NixOS

# Install

See [install.md](install.md)

# Migration from other apps

See [codes_migration.md](codes_migration.md)

## Planned features

Currently there is not any planned feature. If you need something new that could improve the software feel free to open an issue.

## Contribution

I created this project for my own needs, but I would be happy if this little program is useful to someone else, and I gratefully accept any pull requests.
