import sqlite3
import sys
import json


def get_accounts(filename):
    conn = sqlite3.connect(filename)
    c = conn.cursor()
    accounts = []
    for row in c.execute("SELECT email,secret,issuer,type,counter FROM accounts"):
        if row[3] == 0:
            type_ = "TOTP"
        else:
            type_ = "HOTP"
        accounts.append(
            {
                'label': row[0],
                'secret': row[1],
                'issuer': row[2],
                'type': type_,
                'digits': 6,
                'counter': row[4],
                'algorithm': 'SHA1',
            }
        )
    c.close()
    return accounts


def main():
    if len(sys.argv) != 3:
        print("Usage: python gauth.py [INPUT_FILE] [OUTPUT_FILE]")
        return
    output_file = open(sys.argv[2], 'w')
    output_file.write(json.dumps(get_accounts(sys.argv[1])))
    output_file.close()


main()
