"""Je vérifie les fichiers réellement publiés avec la galerie de démonstration."""

import json
from pathlib import Path
import re
import struct
import unittest
from urllib.parse import unquote, urlsplit
import zlib


ROOT = Path(__file__).resolve().parents[1]
DEMO = ROOT / "docs" / "demo"
IMAGES = (
    "01-bilan-mensuel", "02-bilan-hebdomadaire", "03-activites-classes-historique",
    "04-recherche-activite", "05-zones", "06-donjons", "07-duo", "08-trio",
    "09-pl-arene", "10-modifier-session", "11-sauvegarder", "12-charger",
)


class DemoGalleryTests(unittest.TestCase):
    def test_twelve_complete_pngs_have_the_expected_dimensions(self):
        images = DEMO / "images"
        self.assertEqual({p.stem for p in images.glob("*.png")}, set(IMAGES))
        for index, name in enumerate(IMAGES):
            with self.subTest(image=name):
                content = (images / f"{name}.png").read_bytes()
                self.assertEqual(content[:8], b"\x89PNG\r\n\x1a\n")
                expected = (1180, 820) if index >= 10 else (1440, 1600 if index >= 4 else 1100)
                self.assertEqual(struct.unpack(">II", content[16:24]), expected)
                position, compressed, ended = 8, bytearray(), False
                while position < len(content):
                    length = struct.unpack(">I", content[position:position + 4])[0]
                    kind = content[position + 4:position + 8]
                    payload = content[position + 8:position + 8 + length]
                    crc = struct.unpack(">I", content[position + 8 + length:position + 12 + length])[0]
                    self.assertEqual(zlib.crc32(kind + payload) & 0xFFFFFFFF, crc)
                    if kind == b"IDAT":
                        compressed.extend(payload)
                    if kind == b"IEND":
                        ended = True
                        self.assertEqual(position + 12, len(content))
                    position += length + 12
                self.assertTrue(ended)
                # Je contrôle aussi les pixels décompressés, pas uniquement l'en-tête PNG.
                pixels = zlib.decompress(compressed)
                self.assertEqual(len(pixels), expected[1] * (1 + expected[0] * 4))
                self.assertGreater(len(set(pixels)), 100)

    def test_readme_and_gallery_links_resolve_inside_the_repository(self):
        for document in (ROOT / "README.md", DEMO / "README.md"):
            content = document.read_text(encoding="utf-8")
            for target in re.findall(r"!?\[[^\]]*\]\(([^)]+)\)", content):
                parsed = urlsplit(target)
                if parsed.scheme or not parsed.path:
                    continue
                with self.subTest(document=document.name, target=target):
                    path = (document.parent / unquote(parsed.path)).resolve()
                    self.assertTrue(path.is_relative_to(ROOT))
                    self.assertTrue(path.is_file())
            self.assertNotIn("C:\\Users\\", content)
        gallery = (DEMO / "README.md").read_text(encoding="utf-8")
        references = re.findall(r"!\[[^\]]+\]\(images/([^)]*)\.png\)", gallery)
        self.assertEqual(references, list(IMAGES))
        self.assertEqual(len(re.findall(r"^## \d+\.", gallery, re.MULTILINE)), 12)
        self.assertIn('id="importer-la-demo"', gallery)

    def test_downloadable_month_agrees_with_the_published_totals(self):
        state = json.loads((DEMO / "mois-demo.json").read_text(encoding="utf-8"))
        self.assertEqual(state["schema_version"], 2)
        data = state["data"]
        self.assertEqual([len(data[k]) for k in ("zones", "dungeons", "duo_trios", "arenas")], [24, 16, 12, 8])
        totals = [
            sum(e["session_total_kamas"] for e in data["zones"]),
            sum(e["gross_kamas_per_run"] - e["key_price"] for e in data["dungeons"]),
            sum(e["loot_kamas_per_run"] + e["full_soul_sale_price"] - e["capture_stone_price"]
                - e["key_unit_price"] * (2 if e["party_mode"] == "duo" else 3) for e in data["duo_trios"]),
            sum(e["seat_price"] * e["seats_sold"] - e["capture_price"] * e["captures_count"] for e in data["arenas"]),
        ]
        self.assertEqual(totals, [5_561_000, 2_264_000, 2_988_000, 294_000])
        self.assertEqual(sum(totals), 11_107_000)
        gallery = (DEMO / "README.md").read_text(encoding="utf-8")
        for value in totals + [sum(totals)]:
            self.assertIn(f"{int(value):,}".replace(",", " "), gallery)
        dates = sorted(e["recorded_at"][:10] for entries in data.values() for e in entries)
        self.assertEqual(len(set(dates)), 20)
        self.assertEqual((dates[0], dates[-1]), ("2026-08-14", "2026-09-12"))
        self.assertEqual(sum(bool(draft) for draft in state["drafts"].values()), 4)


if __name__ == "__main__":
    unittest.main()
