"""Tests for build.py. Run them with:

    python3 -m unittest discover -s catalog -p 'test_*.py'
"""

import copy
import json
import tempfile
import unittest
from pathlib import Path

import build

BUNDLED = json.loads(build.SOURCE.read_text())


def catalog():
    return copy.deepcopy(BUNDLED)


def compositor(data, compositor_id):
    return next(item for item in data["compositors"] if item["id"] == compositor_id)


class BundledCatalogTests(unittest.TestCase):
    def test_bundled_catalog_is_valid(self):
        self.assertEqual(build.validate(catalog()), [])


class ValidateTests(unittest.TestCase):
    def test_unknown_spotlight_item(self):
        data = catalog()
        data["spotlights"][0]["id"] = "not-a-distro"
        self.assertTrue(any("spotlight" in problem for problem in build.validate(data)))

    def test_rice_on_unknown_compositor(self):
        data = catalog()
        data["rices"][0]["compositors"] = ["not-a-compositor"]
        self.assertTrue(any("rice" in problem for problem in build.validate(data)))

    def test_duplicate_ids(self):
        data = catalog()
        data["distros"].append(copy.deepcopy(data["distros"][0]))
        self.assertTrue(any("appears twice" in problem for problem in build.validate(data)))

    def test_missing_logo_file(self):
        data = catalog()
        data["distros"][0]["logo"] = "no-such-logo"
        self.assertTrue(any("no-such-logo" in problem for problem in build.validate(data)))

    def test_gallery_image_without_credit(self):
        data = catalog()
        compositor(data, "sway")["gallery"] = [{"url": "https://example.org/a.png", "page": "https://example.org/a", "credit": "", "license": "CC0"}]
        self.assertTrue(any("gallery" in problem for problem in build.validate(data)))

    def test_other_schema_version(self):
        data = catalog()
        data["schema_version"] = 2
        self.assertTrue(any("schema_version" in problem for problem in build.validate(data)))


class ProtocolTests(unittest.TestCase):
    def dump(self, timestamp_ms, interfaces, version="9.9"):
        return {
            "generationTimestamp": timestamp_ms,
            "version": version,
            "globals": [{"interface": name, "version": 1} for name in interfaces],
        }

    def test_newer_dump_replaces_the_list(self):
        data = catalog()
        sway = compositor(data, "sway")
        interfaces = sway["protocols"]["globals"] + ["ext_new_protocol_v1"]
        build.apply_protocol_dump(sway, self.dump(4_102_444_800_000, interfaces), build.Report())

        self.assertEqual(sway["protocols"]["version"], "9.9")
        self.assertEqual(sway["protocols"]["checked_on"], "2100-01-01")
        self.assertIn("ext_new_protocol_v1", sway["protocols"]["globals"])

    def test_older_dump_is_ignored(self):
        data = catalog()
        sway = compositor(data, "sway")
        before = copy.deepcopy(sway["protocols"])
        build.apply_protocol_dump(sway, self.dump(0, ["wl_compositor"]), build.Report())
        self.assertEqual(sway["protocols"], before)

    def test_a_dump_that_loses_most_globals_is_ignored(self):
        data = catalog()
        sway = compositor(data, "sway")
        before = copy.deepcopy(sway["protocols"])
        report = build.Report()
        build.apply_protocol_dump(sway, self.dump(4_102_444_800_000, ["wl_compositor"]), report)

        self.assertEqual(sway["protocols"], before)
        self.assertEqual(len(report.warnings), 1)

    def test_a_failed_download_keeps_the_old_data(self):
        data = catalog()
        before = copy.deepcopy(data["compositors"])

        def failing_fetch(url):
            raise OSError("offline")

        report = build.Report()
        build.refresh_protocols(data, json.loads(build.SOURCES_CONFIG.read_text()), report, fetch=failing_fetch)
        self.assertEqual(data["compositors"], before)
        self.assertTrue(report.warnings)


RICES_TS = """
type Contest = { name: string; number: number }

export const contests: Contest[] = [
	{
		name: 'Moon',
		number: 5,
		date: 'June 2026',
		color: colors.indigo[500],
		rices: [
			{
				name: 'Astroland',
				creator: 'zacoons',
				pretitel: '#1',
				dotfilesLink: 'https://codeberg.org/zacoons/astroland_dots',
				thumbnail: '/ricing_competitions/5/zacoons.webp',
				video: 'https://dl.hypr.land/contest-videos/astroland_zacoons.webm'
			},
			{
				name: 'It\\'s Unnamed',
				creator: 'Lyasm',
				pretitel: '#2 (ex aequo)',
				dotfilesLink: '#',
				thumbnail: '/ricing_competitions/5/lyasm.webp'
			}
		]
	},
	{
		name: 'Winter',
		number: 1,
		date: 'December 2022',
		color: colors.sky[500],
		rices: [
			{
				name: 'Aurora',
				creator: 'flick0',
				pretitel: '#1',
				dotfilesLink: 'https://github.com/flick0/dotfiles/tree/aurora',
				thumbnail: '/ricing_competitions/1/flicko.webp'
			}
		]
	}
]
"""
PAGE = "https://hypr.land/hall_of_fame/"


class HallOfFameTests(unittest.TestCase):
    def parse(self):
        return build.parse_hall_of_fame(RICES_TS, "https://hypr.land", PAGE)

    def test_reads_contests_newest_first(self):
        contests = self.parse()
        self.assertEqual([contest["name"] for contest in contests], ["Moon", "Winter"])
        self.assertEqual(contests[0]["number"], 5)
        self.assertEqual(contests[0]["date"], "June 2026")

    def test_reads_each_rice(self):
        astroland = self.parse()[0]["rices"][0]
        self.assertEqual(
            astroland,
            {
                "name": "Astroland",
                "author": "zacoons",
                "place": "#1",
                "image": "https://hypr.land/ricing_competitions/5/zacoons.webp",
                "link": "https://codeberg.org/zacoons/astroland_dots",
            },
        )

    def test_unescapes_quotes_and_links_missing_dotfiles_to_the_page(self):
        unnamed = self.parse()[0]["rices"][1]
        self.assertEqual(unnamed["name"], "It's Unnamed")
        self.assertEqual(unnamed["link"], PAGE)

    def test_a_file_without_contests_is_an_error(self):
        with self.assertRaises(ValueError):
            build.parse_hall_of_fame("export const nothing = []", "https://hypr.land", PAGE)

    def test_fewer_contests_keep_the_old_list(self):
        data = catalog()
        before = copy.deepcopy(data["hall_of_fame"])
        report = build.Report()
        build.apply_hall_of_fame(data["hall_of_fame"], self.parse()[:1], report)
        self.assertEqual(data["hall_of_fame"], before)
        self.assertTrue(report.warnings)

    def test_a_failed_download_keeps_the_old_data(self):
        data = catalog()
        before = copy.deepcopy(data["hall_of_fame"])

        def failing_fetch(url):
            raise OSError("offline")

        report = build.Report()
        build.refresh_hall_of_fame(data, json.loads(build.SOURCES_CONFIG.read_text()), report, fetch=failing_fetch)
        self.assertEqual(data["hall_of_fame"], before)
        self.assertTrue(report.warnings)

    def test_a_gone_screenshot_drops_the_rice(self):
        data = catalog()
        first = data["hall_of_fame"]["contests"][0]["rices"][0]
        build.check_hosted_media(data, build.Report(), status_of=lambda url: "gone" if url == first["image"] else "ok")
        self.assertNotIn(first, data["hall_of_fame"]["contests"][0]["rices"])


class MediaTests(unittest.TestCase):
    def test_a_gone_screenshot_is_dropped(self):
        data = catalog()
        gone_url = compositor(data, "sway")["screenshot"]["url"]
        build.check_hosted_media(data, build.Report(), status_of=lambda url: "gone" if url == gone_url else "ok")
        self.assertNotIn("screenshot", compositor(data, "sway"))

    def test_unknown_status_keeps_the_screenshot(self):
        data = catalog()
        build.check_hosted_media(data, build.Report(), status_of=lambda url: "unknown")
        self.assertIn("screenshot", compositor(data, "sway"))

    def test_a_gone_gallery_image_is_dropped(self):
        data = catalog()
        sway = compositor(data, "sway")
        kept = {"url": "https://example.org/kept.png", "page": "https://example.org/kept", "credit": "A", "license": "CC0"}
        gone = {"url": "https://example.org/gone.png", "page": "https://example.org/gone", "credit": "B", "license": "CC0"}
        sway["gallery"] = [kept, gone]
        build.check_hosted_media(data, build.Report(), status_of=lambda url: "gone" if url == gone["url"] else "ok")
        self.assertEqual(sway["gallery"], [kept])

    def test_too_many_gone_stops_the_build(self):
        with self.assertRaises(SystemExit):
            build.check_hosted_media(catalog(), build.Report(), status_of=lambda url: "gone")


class PublishTests(unittest.TestCase):
    def test_writes_catalog_manifest_and_index(self):
        with tempfile.TemporaryDirectory() as folder:
            out = Path(folder)
            build.publish(catalog(), out, "2026-11-07T04:17:00Z")

            published = json.loads((out / "v1/catalog.json").read_text())
            manifest = json.loads((out / "v1/manifest.json").read_text())
            self.assertEqual(published["generated_at"], "2026-11-07T04:17:00Z")
            self.assertEqual(manifest["schema_version"], build.SCHEMA_VERSION)
            self.assertTrue((out / "index.html").exists())


if __name__ == "__main__":
    unittest.main()
