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
FONT_POOL = None
OLD_FONTS = {}
FAMILIES = {
    "sarabun": "เอกสารและเนื้อหา",
    "notosansthai": "อ่านง่าย ใช้งานทั่วไป",
    "notoserifthai": "บทความและงานทางการ",
    "prompt": "หัวข้อและงานแบรนด์",
    "kanit": "แบนเนอร์และหัวข้อ",
    "anuphan": "หน้าจอและงานสมัยใหม่",
    "baijamjuree": "งานออกแบบและเนื้อหา",
    "ibmplexsansthai": "เว็บไซต์และงานธุรกิจ",
    "athiti": "เนื้อหาโปร่ง อ่านสบาย",
    "chakrapetch": "หัวข้อแนวเทคโนโลยี",
    "charm": "ลายมือไทยและคำเชิญ",
    "charmonman": "ลายมืออ่อนช้อย",
    "chonburi": "หัวข้อเด่น มีเอกลักษณ์",
    "fahkwang": "งานบรรณาธิการและแบรนด์",
    "googlesans": "หน้าจอและงานสมัยใหม่",
    "ibmplexsansthailooped": "ตัวอักษรมีหัวสำหรับเนื้อหา",
    "itim": "ลายมือเป็นกันเอง",
    "k2d": "งานดิจิทัลและหัวข้อ",
    "koho": "งานสร้างสรรค์ร่วมสมัย",
    "kodchasan": "ตัวอักษรโค้งมน",
    "krub": "เนื้อหาและงานธุรกิจ",
    "maitree": "บทความและงานทางการ",
    "mali": "ลายมือน่ารัก ใช้งานสนุก",
    "mitr": "หัวข้อหนา อ่านชัด",
    "niramit": "งานพิมพ์และงานออกแบบ",
    "notosansthailooped": "ตัวอักษรมีหัว อ่านง่าย",
    "pattaya": "หัวข้อแนวลายมือ",
    "playpensansthai": "ลายมือสำหรับงานสร้างสรรค์",
    "pridi": "บทความและเนื้อหายาว",
    "sriracha": "ลายมือไทยที่คุ้นเคย",
    "srisakdi": "ลายมือสำหรับคำเชิญ",
    "taviraj": "งานบรรณาธิการและเอกสาร",
    "thasadith": "เนื้อหาบางเบาและหัวข้อ",
    "trirong": "งานทางการและงานพิมพ์",
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
    def read_font(block):
        filename = re.search(r'filename: "([^"]+)"', block).group(1)
        assert "/" not in filename and filename.endswith(".ttf")
        url = base + urllib.parse.quote(filename, safe="")
        cache_dir = ROOT / ".catalog-cache"
        cache_dir.mkdir(exist_ok=True)
        old_hash = OLD_FONTS.get((slug, filename))
        old_file = cache_dir / (str(old_hash) + ".ttf")
        data = old_file.read_bytes() if old_hash and old_file.exists() else b""
        if not old_hash or hashlib.sha256(data).hexdigest() != old_hash:
            data = fetch(url)
        checksum = hashlib.sha256(data).hexdigest()
        (cache_dir / (checksum + ".ttf")).write_bytes(data)
        return {"filename": filename, "sha256": checksum,
                      "size": len(data), "weight": int(re.search(r'weight: (\d+)', block).group(1)),
                      "italic": 'style: "italic"' in block}
    files = list(FONT_POOL.map(read_font, blocks))
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
    previous = json.loads((ROOT / "core/catalog.json").read_text())
    if previous["revision"] == revision:
        OLD_FONTS = {(f["id"], font["filename"]): font["sha256"]
                     for f in previous["families"] for font in f["files"]}
    with concurrent.futures.ThreadPoolExecutor(max_workers=16) as font_pool:
        FONT_POOL = font_pool
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            families = list(pool.map(lambda slug: family(slug, revision), FAMILIES))
    metadata_url = "https://fonts.google.com/metadata/fonts"
    metadata_text = fetch(metadata_url).decode()
    if metadata_text.startswith(")]}'"):
        metadata_text = metadata_text.split("\n", 1)[1]
    live_families = sorted(f["family"] for f in json.loads(metadata_text)["familyMetadataList"]
                           if "thai" in f.get("subsets", []))
    assert sorted(f["name"] for f in families) == live_families, "Catalog differs from Google's Thai subset; review the family list"
    payload = {"revision": revision, "families": families,
               "thai_metadata_source": metadata_url, "thai_family_names": live_families}
    (ROOT / "core/catalog.json").write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n")
    print(f"Pinned {len(families)} families, {sum(len(f['files']) for f in families)} font files at {revision}")
