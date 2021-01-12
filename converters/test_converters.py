from authy import fetch_json as authy_fetch_json, object_to_cotp_json as authy_converter
from gauth import get_accounts as gauth_converter


def test_gauth_converter():
    assert gauth_converter("converters/example_databases/gauth_db.sqlite3") == [{"label": "SHA512 10", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None, "digits": 6}, {"label": "SHA512", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None, "digits": 6}]

def test_authy_converter():
    assert authy_converter(authy_fetch_json("converters/example_databases/authy_db.xml")) == [{"label": "Test", "secret": "TLQEUBNC4ENYRDMLM2ZMQPN7PE272AW7", "issuer": "", "digits": 6}]