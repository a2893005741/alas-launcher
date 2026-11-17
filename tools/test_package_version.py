"""Validate executable release metadata and complete manifest generation."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from prepare_release import prepare

class PackageVersionTest(unittest.TestCase):
    def test_release_manifest_requires_matching_versions_and_hashes_executable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            (root/'target/release').mkdir(parents=True)
            (root/'Cargo.toml').write_text('[package]\nversion="1.2.3"\n')
            (root/'tauri.conf.json').write_text('{"version":"1.2.3"}')
            (root/'target/release/warshippilot-launcher.exe').write_bytes(b'executable')
            output=root/'release';output.mkdir()
            (output/'windows-x86_64.json').write_text(json.dumps({'schema':2,'revision':'a'*40,'platforms':{'windows-x86_64':[]}}))
            prepare(root,output)
            value=json.loads((output/'stable.json').read_text())
            self.assertEqual(value['version'],'1.2.3')
            self.assertEqual(value['platforms']['windows-x86_64']['sha256'],hashlib.sha256(b'executable').hexdigest())
            self.assertEqual(value['runtime']['revision'],'a'*40)
            (root/'tauri.conf.json').write_text('{"version":"1.2.4"}')
            with self.assertRaises(ValueError):prepare(root,output)
