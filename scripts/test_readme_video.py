"""Je vérifie le lecteur du README et les fichiers vidéo livrés avec le dépôt."""

from html.parser import HTMLParser
from pathlib import Path
import struct
import unittest
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]
RAW_PREFIX = "/Donj63000/Dofus-Retro-EvoFarm-Dev-By-Donj63000-Coca/master/"


class VideoParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.videos = []

    def handle_starttag(self, tag, attrs):
        if tag == "video":
            self.videos.append(dict(attrs))


class ReadmeVideoTests(unittest.TestCase):
    def test_player_references_published_assets_and_requires_user_playback(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        parser = VideoParser()
        parser.feed(readme)
        self.assertEqual(len(parser.videos), 1)
        video = parser.videos[0]
        self.assertIn("controls", video)
        self.assertNotIn("autoplay", video)
        source = urlsplit(video["src"])
        self.assertEqual(source.scheme, "https")
        self.assertEqual(source.netloc, "raw.githubusercontent.com")
        self.assertTrue(source.path.startswith(RAW_PREFIX))
        self.assertTrue((ROOT / source.path.removeprefix(RAW_PREFIX)).is_file())
        self.assertTrue((ROOT / video["poster"]).is_file())
        self.assertIn(f']({video["src"]})', readme)

    def test_mp4_is_complete_and_ready_for_progressive_playback(self):
        content = (ROOT / "docs/demo/videos/EvoFarm-demonstration-4min.mp4").read_bytes()
        position, boxes = 0, {}
        while position < len(content):
            self.assertGreaterEqual(len(content) - position, 8)
            size, kind = struct.unpack_from(">I4s", content, position)
            self.assertGreaterEqual(size, 8)
            self.assertLessEqual(position + size, len(content))
            boxes[kind] = (position, content[position + 8:position + size])
            position += size
        self.assertIn(b"ftyp", boxes)
        self.assertIn(b"moov", boxes)
        self.assertIn(b"mdat", boxes)
        # Je vérifie que les métadonnées précèdent les images pour démarrer sans tout télécharger.
        self.assertLess(boxes[b"moov"][0], boxes[b"mdat"][0])
        metadata = boxes[b"moov"][1]
        self.assertIn(b"avc1", metadata)
        self.assertIn(b"mp4a", metadata)
        self.assertEqual(metadata[4:8], b"mvhd")
        self.assertEqual(metadata[8], 0)
        timescale, duration = struct.unpack_from(">II", metadata, 20)
        self.assertGreater(timescale, 0)
        self.assertAlmostEqual(duration / timescale, 240, delta=0.1)

    def test_subtitles_cover_the_published_video(self):
        subtitles = (ROOT / "docs/demo/videos/EvoFarm-demonstration-fr.srt").read_text(encoding="utf-8")
        self.assertIn("00:00:00,000 -->", subtitles)
        self.assertIn("--> 00:04:00,000", subtitles)
        self.assertEqual(subtitles.count(" --> "), 24)


if __name__ == "__main__":
    unittest.main()
