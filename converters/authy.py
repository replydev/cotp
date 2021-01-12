import sys
import xml.etree.ElementTree as ET
import json

def fetch_json(filename):
    tree = ET.parse(filename)
    j = tree.find("string").text.replace("&quot;","\"")
    return json.loads(j)
    
def object_to_cotp_json(d):
    final = []
    for element in d:
        final.append(
            {
                'label': element['name'],
                'secret': element['decryptedSecret'],
                'issuer': "",
                'digits': int(element['digits'])
            }
        )
    return final

def main():
    if len(sys.argv) != 3:
        print("Usage: python authy.py [INPUT_FILE] [OUTPUT_FILE]")
        return

    data = fetch_json(sys.argv[1])
    output_file = open(sys.argv[2],'w')
    output_file.write(json.dumps(object_to_cotp_json(data)))
    output_file.close()

main()

def test_conversion():
    assert fetch_json("exported_databases/authy_db.xml") == [{"label": "Test", "secret": "TLQEUBNC4ENYRDMLM2ZMQPN7PE272AW7", "issuer": "", "digits": 6}]