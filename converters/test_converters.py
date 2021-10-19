from authy import fetch_json as authy_fetch_json, object_to_cotp_json as authy_converter
from gauth import get_accounts as gauth_converter
from mauth import get_accounts as mauth_converter


def test_gauth_converter():
    assert gauth_converter("converters/example_databases/gauth_db.sqlite3") == [
        {"label": "SHA512 10", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None,
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "SHA512", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None,
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}]


def test_authy_converter():
    assert authy_converter(authy_fetch_json("converters/example_databases/authy_db.xml")) == [
        {"label": "Test", "secret": "TLQEUBNC4ENYRDMLM2ZMQPN7PE272AW7", "issuer": "", "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}]


def test_mauth_converter():
    assert mauth_converter("converters/example_databases/microsoft_authenticator/PhoneFactor") == [
        {"label": "Test1", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test1",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "Test2", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test2",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "Test3", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test3",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}
    ]
