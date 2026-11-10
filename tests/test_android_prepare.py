import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from tools import lab

with patch.dict(sys.modules, {'lab': lab}):
    from tools import android_prepare as prepare


def sha1(data):
    return hashlib.sha1(data).hexdigest()


class AndroidPrepareTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.inputs = self.root / 'official'
        self.inputs.mkdir()
        downloads = {}
        for side in ['client', 'server']:
            data = f'{side} fixture'.encode()
            source = self.inputs / f'{side}.jar'
            source.write_bytes(data)
            downloads[side] = {'url': source.as_uri(), 'sha1': sha1(data)}
        index = self.inputs / 'assets.json'
        index.write_bytes(b'{"objects": {}}')
        self.manifest = {
            'id': '26.3',
            'javaVersion': {'component': 'test', 'majorVersion': 25},
            'downloads': downloads,
            'libraries': [],
            'assetIndex': {'id': '34', 'url': index.as_uri(), 'sha1': sha1(index.read_bytes())},
        }
        self.pins = {
            'version': '26.3',
            'client_sha1': downloads['client']['sha1'],
            'server_sha1': downloads['server']['sha1'],
            'java_major': 25,
        }
        self.destination = self.root / 'minecraft'
        for attribute, value in [
            ('ROOT', self.root),
            ('MANIFEST', self.inputs.as_uri() + '/{checksum}/{version}.json'),
        ]:
            replacement = patch.object(prepare, attribute, value)
            replacement.start()
            self.addCleanup(replacement.stop)

    def publish_manifest(self):
        data = json.dumps(self.manifest).encode()
        self.pins['manifest_sha1'] = sha1(data)
        self.lock = {'minecraft': self.pins}
        (self.root / 'research-lock.json').write_text(json.dumps(self.lock))
        path = self.inputs / self.pins['manifest_sha1'] / '26.3.json'
        path.parent.mkdir()
        path.write_bytes(data)
        return path

    def test_preparation_downloads_the_pinned_manifest_and_verified_inputs(self):
        source = self.publish_manifest()
        # Exercise the real downloader with local fixtures; no mutable catalog is needed.
        manifest, server = prepare.download_game('26.3', self.destination)
        version = self.destination / 'versions/26.3'
        self.assertEqual((version / '26.3.json').read_bytes(), source.read_bytes())
        self.assertEqual((version / '26.3.jar').read_bytes(), b'client fixture')
        self.assertEqual(server.read_bytes(), b'server fixture')
        self.assertEqual(manifest, self.manifest)
        # The existing preparation acceptance check must accept what Android downloads.
        accepted = lab.import_manifest(self.lock, self.root / 'prepared', self.destination)
        self.assertEqual(accepted, manifest)

    def test_a_manifest_with_the_wrong_checksum_is_not_published(self):
        source = self.publish_manifest()
        source.write_bytes(b'{"id": "tampered"}')
        with self.assertRaisesRegex(ValueError, 'Checksum mismatch'):
            prepare.download_game('26.3', self.destination)
        self.assertFalse((self.destination / 'versions/26.3/26.3.json').exists())

    def test_jar_pins_still_reject_unreviewed_inputs(self):
        self.manifest['downloads']['client']['sha1'] = '0' * 40
        self.publish_manifest()
        with self.assertRaisesRegex(ValueError, 'client jar differs from the engine pin'):
            prepare.download_game('26.3', self.destination)
        self.assertFalse((self.destination / 'versions/26.3/26.3.jar').exists())


if __name__ == '__main__':
    unittest.main()
