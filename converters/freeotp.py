import base64
import sys
import xml.etree.ElementTree as Et
import json


def fetch_json(filename):
    element_list = []
    tree = Et.parse(filename)
    for e in tree.findall("string"):
        element_list.append(json.loads(e.text.replace("&quot;", "\"")))
    return element_list


def object_to_cotp_json(d):
    final = []
    for element in d:
        if type(element) is dict:
            final.append(
                {
                    'label': element['label'],
                    'secret': byte_array_to_base32(element['secret']),
                    'issuer': element['issuerExt'],
                    'algorithm': element['algo'],
                    'type': element['type'],
                    'digits': int(element['digits']),
                    'counter': element['counter'],
                }
            )
    return final

def byte_array_to_base32(secret: list):
    b = bytearray()
    for n in secret:
        # convert signed byte to unsigned
        b.append(n & 255)

    return base64.b32encode(b).decode('utf-8')

def main():
    if len(sys.argv) != 3:
        print("Usage: python authy.py [INPUT_FILE] [OUTPUT_FILE]")
        return

    data = fetch_json(sys.argv[1])
    output_file = open(sys.argv[2], 'w')
    output_file.write(json.dumps(object_to_cotp_json(data)))
    output_file.close()


main()
