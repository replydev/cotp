import hashlib
from pathlib import Path
import re
import sys
import requests


COTP_PKGBUILD_TEMPLATE_PATH = "ci/templates/PKGBUILD.cotp.template"
COTP_BIN_PKGBUILD_TEMPLATE_PATH = "ci/templates/PKGBUILD.cotp-bin.template"

COTP_PKGBUILD_PATH = "ci/cotp/PKGBUILD"
COTP_BIN_PKGBUILD_PATH = "ci/cotp-bin/PKGBUILD"


def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)


def download_file(url: str, path: str):
    r = requests.get(url, stream=True)
    with open(path, "wb") as f:
        for buffer in r.iter_content(chunk_size=16 * 1024):
            f.write(buffer)


def replace(path: str, destination_path: str, version: str, digest: str):
    f = Path(path)
    content = f.read_text()
    wf = Path(destination_path)
    wf.write_text(
        content.replace("%AUR_PKG_VERSION%", version).replace(
            "%AUR_SRC_SHA256SUM%", digest
        )
    )


def file_digest(path: str):
    digest = None
    with open(path, "rb") as f:
        digest = hashlib.sha256(f.read()).hexdigest()
    return digest


def main():
    if len(sys.argv) != 2:
        eprint("Usage: python script.py <VERSION>")
        return

    rawVersion = sys.argv[1]
    match = re.search(r"^v?([0-9]\.[0-9]\.[0-9])$", rawVersion)

    if match is None:
        eprint(f"Invalid version: {rawVersion}")
        return

    version = match.group(1)
    print(f"Version detected: {version}")
    Path("ci/cotp").mkdir(exist_ok=True)
    Path("ci/cotp-bin").mkdir(exist_ok=True)

    source_url = f"https://github.com/replydev/cotp/archive/v{version}.tar.gz"
    source_filename = f"{version}.tar.gz"
    download_file(source_url, source_filename)
    source_digest = file_digest(source_filename)
    print(f"Source digest: {source_digest}")

    compiled_bin_url = f"https://github.com/replydev/cotp/releases/download/v{version}/cotp-v{version}-x86_64-linux.tar.xz"
    compiled_bin_filename = f"cotp-v{version}-x86_64-linux.tar.xz"
    download_file(compiled_bin_url, compiled_bin_filename)
    compiled_bin_digest = file_digest(compiled_bin_filename)
    print(f"Binary digest: {compiled_bin_digest}")

    replace(COTP_PKGBUILD_TEMPLATE_PATH, COTP_PKGBUILD_PATH, version, source_digest)
    replace(
        COTP_BIN_PKGBUILD_TEMPLATE_PATH,
        COTP_BIN_PKGBUILD_PATH,
        version,
        compiled_bin_digest,
    )


if __name__ == "__main__":
    main()
