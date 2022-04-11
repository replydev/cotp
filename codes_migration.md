# TOTP Codes migration from other apps
cotp supports TOTP codes migration from various apps.
Some needs to be converted using simple python script you can find listed in the table below.


| App | How to fetch backup | Needs conversion | cotp argument |
|--|--|--|--|
| [andOTP](https://github.com/andOTP/andOTP) | Make a backup using the app itself. | No |`--andotp` |
| [Aegis](https://github.com/beemdevelopment/Aegis) | Make a backup using the app itself. | No |`--aegis` |
| [Authy](https://authy.com/) | Obtain `/data/data/com.authy.authy/shared_prefs/com.authy.storage.tokens.authenticator.xml` from your phone. | [Yes](https://github.com/replydev/cotp/blob/master/converters/authy.py) | `--authy` |
| [Authy](https://authy.com/) (2nd method) | Follow this guide: https://gist.github.com/gboudreau/94bb0c11a6209c82418d01a59d958c93. | No | `--authy-exported` |
| [cotp](https://github.com/replydev/cotp) | Export you database using `cotp export`. | No |`--cotp` |
| [FreeOTP](https://freeotp.github.io/) | Obtain `/data/data/org.fedorahosted.freeotp/shared_prefs/tokens.xml` from your phone. | [Yes](https://github.com/replydev/cotp/blob/master/converters/freeotp.py) | `--freeotp` |
| [FreeOTP+](https://github.com/helloworld1/FreeOTPPlus) | Make a backup using the app itself. | No | `--freeotp-plus` |
| [Google Authenticator](https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2) | Obtain `/data/data/com.google.android.apps.authenticator2/databases/databases` from your phone | [Yes](https://github.com/replydev/cotp/blob/master/converters/gauth.py) | `--google-authenticator` |
| [Microsoft Authenticator](https://play.google.com/store/apps/details?id=com.azure.authenticator) | Obtain `/data/data/com.azure.authenticator/databases/PhoneFactor` from your phone. Take also `PhoneFactor-wal`, `PhoneFactor-shm` if they exist in the same folder. | [Yes](https://github.com/replydev/cotp/blob/master/converters/mauth.py) | `--microsoft-authenticator` |

## How to convert
Once you got the correct files run the right python script located in the **converters/** folder in this source code.

**Example:**
`python authy.py path/to/database.xml converted.json`

It will convert the database in a json format readable by cotp.

To terminate the import:
`cotp import --authy --path path/to/converted_database.json`
