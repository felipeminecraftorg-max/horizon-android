"""Only the original offline preview host may pass the Java-artifact boundary."""
import contextlib
import io
import unittest
from unittest import mock
from tools import check_public_tree as guard


class NexoPublicationTests(unittest.TestCase):
    def test_exception_is_one_exact_source_not_a_preview_directory(self):
        self.assertFalse(guard.prohibited(guard.ANDROID_PREVIEW_BRIDGE))
        for path in [
            'experiments/jade-lab/android/src/main/java/dev/horizon/jadelab/OtherActivity.java',
            'experiments/nexo-preview/android/src/main/java/net/minecraft/Client.java',
            'experiments/nexo-preview/private.jar',
            '.local/' + guard.ANDROID_PREVIEW_BRIDGE,
        ]:
            self.assertTrue(guard.prohibited(path), path)

    def test_current_source_must_keep_the_original_harness_header(self):
        for header, expected in [(guard.ANDROID_PREVIEW_HEADER, 0), ('package net.minecraft;', 1)]:
            with self.subTest(header=header), \
                 mock.patch.object(guard, 'paths_in_repository', return_value=[guard.ANDROID_PREVIEW_BRIDGE]), \
                 mock.patch('builtins.open', mock.mock_open(read_data=header+'\n')), \
                 mock.patch('sys.argv', ['check_public_tree.py']), \
                 contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(guard.main(), expected)


if __name__ == '__main__':
    unittest.main()
