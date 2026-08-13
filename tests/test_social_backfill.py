import importlib.util
import sys
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "scripts" / "backfill_social_observations.py"
SPEC = importlib.util.spec_from_file_location("social_backfill", SCRIPT)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class SocialBackfillTest(unittest.TestCase):
    def test_summary_quote_is_not_promoted_to_direct_quote(self):
        text = "2026/5/8 DIARY\n複数人から突然「魔王が来た」と言われ、驚いた。"
        items = list(MODULE.iter_candidates(text, "summary_derived"))

        self.assertEqual(len(items), 1)
        item = items[0]
        self.assertEqual(item.occurred_at, "2026-05-08")
        self.assertEqual(item.observation_type, "paraphrase")
        self.assertIsNone(item.quote_text)
        self.assertEqual(item.quote_candidate, "魔王が来た")
        self.assertTrue(item.needs_raw_verification)
        self.assertFalse(item.is_public)

    def test_raw_transcript_can_preserve_existing_quote(self):
        text = "2026-05-08\n友人に「詳しすぎる」と言われた。"
        items = list(MODULE.iter_candidates(text, "raw_transcript"))

        self.assertEqual(len(items), 1)
        item = items[0]
        self.assertEqual(item.observation_type, "direct_quote")
        self.assertEqual(item.quote_text, "詳しすぎる")
        self.assertIsNone(item.paraphrase)
        self.assertFalse(item.needs_raw_verification)
        self.assertEqual(item.speaker_label, "unknown")

    def test_plain_conversation_is_not_a_social_observation(self):
        text = "2026-05-08\n「今日は暑いね」と話して旅行の相談をした。"
        self.assertEqual(list(MODULE.iter_candidates(text, "raw_transcript")), [])


if __name__ == "__main__":
    unittest.main()
