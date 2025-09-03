## [1.9.7](https://github.com/replydev/cotp/compare/v1.9.6...v1.9.7) (2025-09-03)


### Bug Fixes

* `mismatched_lifetime_syntaxes` & clippy compiler warnings ([#584](https://github.com/replydev/cotp/issues/584)) ([fd6f717](https://github.com/replydev/cotp/commit/fd6f717a43b342bf7fcbce0c3b2668d9f5fc2022))

## [1.9.6](https://github.com/replydev/cotp/compare/v1.9.5...v1.9.6) (2025-06-02)


### Bug Fixes

* do not throw error on empty otp database, in list subcommand ([0222faf](https://github.com/replydev/cotp/commit/0222fafe1a855a7ad41e04a09084ebde223c5874))
* do not throw error on empty otp database, in list subcommand ([#568](https://github.com/replydev/cotp/issues/568)) ([1c921ef](https://github.com/replydev/cotp/commit/1c921efcdf57e74bd041bc7ec9142109d0245b92))

## [1.9.5](https://github.com/replydev/cotp/compare/v1.9.4...v1.9.5) (2025-05-17)


### Bug Fixes

* fix code format when using 10 digits ([45b4a5f](https://github.com/replydev/cotp/commit/45b4a5f65675939b45ffc6e1a3643f331bac6279))
* handle integer underflow on code format ([71d3933](https://github.com/replydev/cotp/commit/71d393392f73f005b0721f2fd0b482992e7f5111))
* update allowed digits range from command-line interface ([b167d02](https://github.com/replydev/cotp/commit/b167d0275e6eba77b2259ced99efb3e519a79862))

## [1.9.4](https://github.com/replydev/cotp/compare/v1.9.3...v1.9.4) (2025-04-08)


### Bug Fixes

* decode otp uri before parsing ([f5f974d](https://github.com/replydev/cotp/commit/f5f974dc767a84350e6b5610615f71c948711fd4))

## [1.9.3](https://github.com/replydev/cotp/compare/v1.9.2...v1.9.3) (2025-04-06)


### Bug Fixes

* implement first glob matching logic for extract subcommand ([2422c9d](https://github.com/replydev/cotp/commit/2422c9d32ca08c17f793d5d67488ee54ff96012e))
* make matching case insensitive again ([0d564ee](https://github.com/replydev/cotp/commit/0d564ee911c954c2c16a751bc7e660a7a3558521))

## [1.9.2](https://github.com/replydev/cotp/compare/v1.9.1...v1.9.2) (2024-11-04)


### Bug Fixes

* cargo clippy ([2c7e601](https://github.com/replydev/cotp/commit/2c7e601e83499a129c64daf3ff3cc197712c8fe4))
* cargo fmt ([e2c3123](https://github.com/replydev/cotp/commit/e2c3123a91a694559663bd80690a9c56cdae70d9))
* **keymaps:** some kind of vim-alike keymaps ([ef0cb98](https://github.com/replydev/cotp/commit/ef0cb98d2ea807632535d299e5880f5dbbc211b2))
* **keymaps:** some kind of vim-alike keymaps ([#510](https://github.com/replydev/cotp/issues/510)) ([0ec3a47](https://github.com/replydev/cotp/commit/0ec3a47159fc4d2a0d6bed7cfe6181468bda9b9a))
* **reverts:** results for the future ([a702acb](https://github.com/replydev/cotp/commit/a702acb50f2d7b053659dce9a9ca5216d12211aa))

## [1.9.1](https://github.com/replydev/cotp/compare/v1.9.0...v1.9.1) (2024-09-12)


### Bug Fixes

* prompt for save if user modifies the HOTP counter ([8568771](https://github.com/replydev/cotp/commit/8568771c91d5c45df2e68f661c6bae4d147b5b6f))

# [1.9.0](https://github.com/replydev/cotp/compare/v1.8.0...v1.9.0) (2024-08-31)


### Features

* add --database-path argument ([9fafde3](https://github.com/replydev/cotp/commit/9fafde3769b6fe87f4fb8ed39439d86a074c173b))
* add --database-path argument ([#485](https://github.com/replydev/cotp/issues/485)) ([e32600c](https://github.com/replydev/cotp/commit/e32600c5a693a8cfd4887c052bd2aace32c94c46))
* use new argument to be able to set database path by commandline ([effd184](https://github.com/replydev/cotp/commit/effd1848295e43426c715c3f2cb3b275a6015d88))


### Performance Improvements

* resolve clippy lints ([565d452](https://github.com/replydev/cotp/commit/565d4520ede6a5d9053445380f6a8079e0419f6c))

# [1.8.0](https://github.com/replydev/cotp/compare/v1.7.3...v1.8.0) (2024-07-29)


### Bug Fixes

* parse user import correctly in delete subcommand ([8ed354f](https://github.com/replydev/cotp/commit/8ed354f8df90e63494594e66f05afa4d27226f22))


### Features

* implement delete subcommand ([4d74db8](https://github.com/replydev/cotp/commit/4d74db8eed0f04c3f8788fe550a93d14e212d9f0))

## [1.7.3](https://github.com/replydev/cotp/compare/v1.7.2...v1.7.3) (2024-07-24)


### Bug Fixes

* optimize table width on list subcommand ([c182195](https://github.com/replydev/cotp/commit/c1821958c92dbd084a4e25dcc358b32dacbab8b9))
* start code index at one as done in dashboard and in edit subcommand ([c90e910](https://github.com/replydev/cotp/commit/c90e910ee6b98529efa5d4beae0a3579a95a9122))
* start code index from one as done in dashboard and in edit subcommand ([#458](https://github.com/replydev/cotp/issues/458)) ([6a36c8d](https://github.com/replydev/cotp/commit/6a36c8def24a36ccc5416ebe0d3063f7e888e5cf)), closes [#457](https://github.com/replydev/cotp/issues/457)

## [1.7.2](https://github.com/replydev/cotp/compare/v1.7.1...v1.7.2) (2024-07-08)


### Bug Fixes

* manipulate secret case by otp_type ([1bcfb17](https://github.com/replydev/cotp/commit/1bcfb172c32bfeacb1b1eabe45f10474fe67866c))
* use builder pattern to build and validate OTPElement ([75242c3](https://github.com/replydev/cotp/commit/75242c3dd6cddd9e95fdce4ab0016fdec2ae73d3))

## [1.7.1](https://github.com/replydev/cotp/compare/v1.7.0...v1.7.1) (2024-06-19)


### Bug Fixes

* check for integer overflow if digits value is too high ([95cf5a2](https://github.com/replydev/cotp/commit/95cf5a28eeebb73b4aab3eb80787a614fe329da4))
* validate digits input value from command line ([d03d4b8](https://github.com/replydev/cotp/commit/d03d4b8da72c25a6cab97b8f91a8ea28a7341bbb))

# [1.7.0](https://github.com/replydev/cotp/compare/v1.6.1...v1.7.0) (2024-05-09)


### Features

* add new List subcommand ([35673f2](https://github.com/replydev/cotp/commit/35673f2182152fc8384674070ed1c87fab288081))
* first draft implementation of List subcommand ([432f428](https://github.com/replydev/cotp/commit/432f42883f5627d46284141f0077b7474d6f92d1))
* implement json list output ([163b839](https://github.com/replydev/cotp/commit/163b83951c5a766e005106b684ccfc3d285c384c))

## [1.6.1](https://github.com/replydev/cotp/compare/v1.6.0...v1.6.1) (2024-05-05)


### Bug Fixes

* Misc OTP Uri import changes ([#425](https://github.com/replydev/cotp/issues/425)) ([cbe2c72](https://github.com/replydev/cotp/commit/cbe2c72720c97e53426aba6d8e2cf94abec7af18))
* show useful errors when import file parsing fails ([81ffa60](https://github.com/replydev/cotp/commit/81ffa606733a24e1cf060cace938adad7758bdbe))

# [1.6.0](https://github.com/replydev/cotp/compare/v1.5.0...v1.6.0) (2024-05-02)


### Features

* implement otp uri copying from QRCode page ([1120f07](https://github.com/replydev/cotp/commit/1120f070b09ea2ac6e86b9507714b881c768ac53))

# [1.5.0](https://github.com/replydev/cotp/compare/v1.4.5...v1.5.0) (2024-03-12)


### Features

* add new extract subcommand and password-stdin flag ([e033e60](https://github.com/replydev/cotp/commit/e033e60d9a0f0df64fd5935bfa43666ded1ee48a))
* implement code extraction ([87d8ccd](https://github.com/replydev/cotp/commit/87d8ccd4e10554ebb81839bebc189384ffec24a4))
* implement password reading from stdin ([0f3bef7](https://github.com/replydev/cotp/commit/0f3bef7d00fb9094e16b9f45b938ca25667ec54f))
* parametrize copy to clipboard and do not set the default ([794a378](https://github.com/replydev/cotp/commit/794a3788f860f83a7d63c4897e79214ae7cc68e0))

## [1.4.5](https://github.com/replydev/cotp/compare/v1.4.4...v1.4.5) (2024-02-22)


### Bug Fixes

* :rocket: remove otp-uri argument as now otp uris can be imported ([5cb20ae](https://github.com/replydev/cotp/commit/5cb20aeec8a04aa6d26b33df266cdedec4422e68))
* :rocket: remove otp-uri argument as now otp uris can be imported ([#397](https://github.com/replydev/cotp/issues/397)) ([50e647e](https://github.com/replydev/cotp/commit/50e647eada1a2df4af0103a77dd78b3fa31dd42a))

## [1.4.4](https://github.com/replydev/cotp/compare/v1.4.3...v1.4.4) (2024-02-16)


### Bug Fixes

* add changelog semantic release dependency ([9e9f151](https://github.com/replydev/cotp/commit/9e9f151a4680b737405de70b89ae9139d361dffb))
* fix build archive and release name ([0798f1e](https://github.com/replydev/cotp/commit/0798f1e2053bc6593e13eb08dff8f4a573b055ee))
* latest CD fixes ([2b6cc8a](https://github.com/replydev/cotp/commit/2b6cc8abd8c5c35d1bd91d5c48451bff0e15e828))
