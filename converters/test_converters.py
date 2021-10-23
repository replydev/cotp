from authy import fetch_json as authy_fetch_json, object_to_cotp_json as authy_converter
from gauth import get_accounts as gauth_converter
from mauth import get_accounts as mauth_converter
from freeotp import fetch_json as freeotp_fetch_json, object_to_cotp_json as freeotp_converter


def test_gauth_converter():
    assert gauth_converter("example_databases/gauth.sqlite3") == [
        {"label": "SHA512 10", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None,
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "SHA512", "secret": "V24KSXF7TXJFGCKG35ZHQVQB4XZK26XPD7UCXU5GXU3POBTXAFLD47SA", "issuer": None,
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}]


def test_authy_converter():
    assert authy_converter(authy_fetch_json("example_databases/authy.xml")) == [
        {"label": "Test", "secret": "TLQEUBNC4ENYRDMLM2ZMQPN7PE272AW7", "issuer": "", "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}]


def test_mauth_converter():
    assert mauth_converter("example_databases/microsoft_authenticator/PhoneFactor") == [
        {"label": "Test1", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test1",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "Test2", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test2",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"},
        {"label": "Test3", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "Test3",
         "digits": 6, "type": "TOTP", "counter": 0, "algorithm": "SHA1"}
    ]

def test_freeotp_converter():
    assert freeotp_converter(freeotp_fetch_json("example_databases/freeotp.xml")) == [
        {"label": "TOTP512", "secret": "3ZUNXX2SU6RZP4QFMS32YAILJFS2I2T2UZXYSHSX7IIMPAQZAC753NG2", "issuer": "", "algorithm": "SHA512", "type": "TOTP", "digits": 6, "counter": 0}, 
        {"label": "TOTP256", "secret": "T2POIVOD6PZYO5YUNIULXIXB3SNXT4GYLSLO5FUNLIXOEMQHY2R6QAGB", "issuer": "", "algorithm": "SHA256", "type": "TOTP", "digits": 8, "counter": 0}, 
        {"label": "HOTP1", "secret": "QEXCCTBU5DXXSFZQIYQAZ7BZUV5TM4EAXMDGSN4TPMR5BSFTFVEXWPMX", "issuer": "", "algorithm": "SHA1", "type": "HOTP", "digits": 8, "counter": 1}, 
        {"label": "Test3", "secret": "XHRHVLZKO5YARFKEH65RFC7NVOOQCZXNHSCB2Y32OCV32ITSQOVV3DSD", "issuer": "", "algorithm": "SHA256", "type": "TOTP", "digits": 8, "counter": 0}, 
        {"label": "TOTP1", "secret": "MC6SGRDYDMCZBEROCAESWB4KBNTGN76C3NPUNAZMIA5G2CNHRJBB7L6W", "issuer": "", "algorithm": "SHA1", "type": "TOTP", "digits": 8, "counter": 0}
        ]
