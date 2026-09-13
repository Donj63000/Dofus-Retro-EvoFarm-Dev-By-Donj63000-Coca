"""Je vérifie l'intégration native de la vidéo dans le README et ses sous-titres."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[1]


class ReadmeVideoTests(unittest.TestCase):
    def test_video_uses_a_standalone_github_attachment_with_fallback_link(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        # Je garde l'URL seule dans son paragraphe : GitHub la transforme en lecteur.
        attachments = re.findall(
            r"\n\n(https://github\.com/user-attachments/assets/[0-9a-f-]{36})\n\n",
            readme,
        )
        self.assertEqual(len(attachments), 1)
        self.assertIn(f"]({attachments[0]})", readme)
        self.assertNotIn("<video", readme)
        self.assertIn('href="#evofarm-en-vidéo"', readme)
        self.assertLess(readme.index("## EvoFarm en vidéo"), readme.index("## Télécharger et commencer"))

    def test_subtitles_exist_and_cover_four_minutes_without_gaps(self):
        relative = "docs/demo/videos/EvoFarm-demonstration-fr.srt"
        self.assertIn(f"]({relative})", (ROOT / "README.md").read_text(encoding="utf-8"))
        subtitles = (ROOT / relative).read_text(encoding="utf-8")
        cues = subtitles.strip().split("\n\n")
        end = 0
        for number, cue in enumerate(cues, 1):
            lines = cue.splitlines()
            self.assertEqual(int(lines[0]), number)
            timestamps = lines[1].split(" --> ")
            self.assertEqual(len(timestamps), 2)
            values = []
            for timestamp in timestamps:
                h, m, s, ms = map(int, re.split(r"[:,]", timestamp))
                values.append(((h * 60 + m) * 60 + s) * 1000 + ms)
            self.assertEqual(values[0], end)
            self.assertGreater(values[1], values[0])
            self.assertTrue("".join(lines[2:]).strip())
            end = values[1]
        self.assertEqual(end, 240_000)


if __name__ == "__main__":
    unittest.main()
