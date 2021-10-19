import sqlite3
import sys
import json
import base64

TOTP_TYPE = 0
MICROSOFT_TYPE = 1


def get_accounts(filename):
    conn = sqlite3.connect(filename)
    c = conn.cursor()
    accounts = []
    for row in c.execute("SELECT account_type,name,username,oath_secret_key FROM accounts"):
        account_type = row[0]
        if account_type == TOTP_TYPE:
            secret = row[3].replace("-", "").replace(" ", "").upper()
        else:
            secret = base64.b32encode(base64.b64decode(row[3]))
        accounts.append(
            {
                'label': row[2],
                'secret': secret,
                'issuer': row[1],
                'digits': 6,
                'type': 'TOTP',
                'counter': 0,
                'algorithm': "SHA1",
            }
        )
    c.close()
    return accounts


def main():
    if len(sys.argv) != 3:
        print("Usage: python mauth.py [INPUT_FILE] [OUTPUT_FILE]")
        return
    # If you got three files, PhoneFactor, PhoneFactor-shm, PhoneFactor-wal just use PhoneFactor
    output_file = open(sys.argv[2], 'w')
    output_file.write(json.dumps(get_accounts(sys.argv[1])))
    output_file.close()


main()
