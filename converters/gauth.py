import sqlite3
import sys
import json

def get_accounts(filename):
    conn = sqlite3.connect(filename)
    c = conn.cursor()
    accounts = []
    for row in c.execute("SELECT email,secret,issuer FROM accounts"):
        accounts.append(
            {
                'label': row[0],
                'secret': row[1],
                'issuer': row[2],
                'digits': 6,
            }
        )
    c.close()
    return accounts

def main():
    if len(sys.argv) != 3:
        print("Usage: python gauth.py [INPUT_FILE] [OUTPUT_FILE]")
        return
    output_file = open(sys.argv[2],'w')
    output_file.write(json.dumps(get_accounts(sys.argv[1])))
    output_file.close()

main()