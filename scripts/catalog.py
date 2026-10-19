"""Maintainer tool: pin Google Fonts sources, licences and SHA-256; no API key.

Run: python3 scripts/catalog.py [40-character Google Fonts commit SHA]
Network access is required only while rebuilding the catalog.
"""
import concurrent.futures
import hashlib
import json
import pathlib
import re
import sys
import time
import urllib.parse
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
FAMILIES = {
    "sarabun": "เอกสารและเนื้อหา",
    "notosansthai": "อ่านง่าย ใช้งานทั่วไป",
    "notoserifthai": "บทความและงานทางการ",
    "prompt": "หัวข้อและงานแบรนด์",
    "kanit": "แบนเนอร์และหัวข้อ",
    "anuphan": "หน้าจอและงานสมัยใหม่",
    "baijamjuree": "งานออกแบบและเนื้อหา",
    "ibmplexsansthai": "เว็บไซต์และงานธุรกิจ",
}


def fetch(url):
    for attempt in range(3):
        try:
            req = urllib.request.Request(url, headers={"User-Agent": "ThaiFontKit-catalog/0.1"})
            with urllib.request.urlopen(req, timeout=60) as response:
                return response.read()
        except Exception:
            if attempt == 2:
                raise
            time.sleep(attempt + 1)


def family(slug, revision):
    base = f"https://raw.githubusercontent.com/google/fonts/{revision}/ofl/{slug}/"
    metadata = fetch(base + "METADATA.pb").decode()
    assert re.search(r'subsets: "thai"', metadata), f"{slug}: missing Thai subset"
    assert re.search(r'license: "OFL"', metadata), f"{slug}: review licence"
    name = re.search(r'^name: "([^"]+)"', metadata).group(1)
    blocks = re.findall(r'fonts \{(.*?)\n\}', metadata, re.S)
    files = []
    for block in blocks:
        filename = re.search(r'filename: "([^"]+)"', block).group(1)
        assert "/" not in filename and filename.endswith(".ttf")
        url = base + urllib.parse.quote(filename, safe="")
        data = fetch(url)
        files.append({"filename": filename, "sha256": hashlib.sha256(data).hexdigest(),
                      "size": len(data), "weight": int(re.search(r'weight: (\d+)', block).group(1)),
                      "italic": 'style: "italic"' in block})
    files.sort(key=lambda f: (f["italic"], abs(f["weight"] - 400), f["weight"]))
    licence = fetch(base + "OFL.txt").decode()
    assert "SIL OPEN FONT LICENSE" in licence.upper()
    return {"id": slug, "name": name, "description": FAMILIES[slug],
            "category": re.search(r'category: "([^"]+)"', metadata).group(1),
            "license": licence, "files": files}


if __name__ == "__main__":
    revision = sys.argv[1] if len(sys.argv) > 1 else json.loads(fetch(
        "https://api.github.com/repos/google/fonts/commits/main"))["sha"]
    assert re.fullmatch("[0-9a-f]{40}", revision)
    with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
        families = list(pool.map(lambda slug: family(slug, revision), FAMILIES))
    payload = {"revision": revision, "families": families}
    (ROOT / "core/catalog.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
    print(f"Pinned {len(families)} families, {sum(len(f['files']) for f in families)} font files at {revision}")
