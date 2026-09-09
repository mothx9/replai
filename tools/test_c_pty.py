#!/usr/bin/env python3
"""Regression controls for final trace delivery racing process exit."""
import unittest
from types import SimpleNamespace
from c_pty import Console


class ExitOrdering(unittest.TestCase):
    def console(self, final):
        console = Console.__new__(Console)
        console.p = SimpleNamespace(poll=lambda: 0, returncode=0)
        console.events = []
        console.errors = bytearray()
        console.bytes = bytearray()

        def drain(_):
            console.events.extend(final)
            final.clear()

        console.pump = drain
        return console

    def test_final_event_is_read_before_exit_is_failure(self):
        event = {'label': 'EVENT', 'kind': 3}
        console = self.console([event])
        self.assertEqual(console.wait('EVENT', kind=3), event)

    def test_missing_or_wrong_event_still_fails(self):
        for final in ([], [{'label': 'EVENT', 'kind': 2}]):
            with self.subTest(final=final):
                with self.assertRaises(AssertionError):
                    self.console(final).wait('EVENT', kind=3)

    def test_prior_event_does_not_satisfy_new_wait(self):
        console = self.console([])
        console.events.append({'label': 'EVENT', 'kind': 3})
        with self.assertRaises(AssertionError):
            console.wait('EVENT', after=1, kind=3)


if __name__ == '__main__':
    unittest.main()
