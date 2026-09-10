#!/usr/bin/env python3
"""Observer regression: arbitrary pipe fragments are not complete state receipts."""
import unittest
from embedding_pty import Reactor


class FragmentedObserver(unittest.TestCase):
    def test_edit_waits_for_new_complete_record(self):
        for initial in [b'', b'FDS 6\nSTATE true 1 1 78\n']:
            for split in range(1, len(b'FDS 6\nSTATE true 1 2 78\n')):
                class Host:
                    receipts = initial
                    state = Reactor.state
                    def send(self, data):
                        self.sent = data
                    def until(self, predicate):
                        record = b'FDS 6\nSTATE true 1 2 78\n'
                        self.receipts += record[:split]
                        assert not predicate(), 'observer accepted an old or partial state'
                        self.receipts += record[split:]
                        assert predicate()
                host = Host()
                state = Reactor.edit(host, b'x', 'x', 1)
                self.assertEqual((host.sent, state['advances']), (b'x', 2))


if __name__ == '__main__':
    unittest.main()
