import sys
import xml.etree.ElementTree as Et
import json


def fetch_json(filename):
    tree = Et.parse(filename)
    j = tree.find("string").text.replace("&quot;", "\"")
    return json.loads(j)


def object_to_cotp_json(d):
    return [
        {
            'label': element['name'],
            'secret': element['decryptedSecret'],
            'issuer': "",
            'type': "TOTP",
            'digits': int(element['digits']),
            'counter': 0,
            "algorithm": "SHA1",
        } for element in d
    ]


def main():
    if len(sys.argv) != 3:
        print("Usage: python authy.py [INPUT_FILE] [OUTPUT_FILE]")
        return

    data = fetch_json(sys.argv[1])
    output_file = open(sys.argv[2], 'w')
    output_file.write(json.dumps(object_to_cotp_json(data)))
    output_file.close()


main()
