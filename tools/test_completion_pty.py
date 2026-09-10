#!/usr/bin/env python3
"""A command result is not a reopened editor: test the observer's input barrier."""
import unittest
from completion_pty import session_reopened

class ReopenedSession(unittest.TestCase):
    def test_every_fragment_waits_for_the_new_prompt(self):
        output=b'demo> bu\r\nhost received: bundle\r\n\x1b[?2004h\r\x1b[2Kdemo> '
        for end in range(len(output)):
            self.assertFalse(session_reopened(output[:end]),end)
        self.assertTrue(session_reopened(output))
    def test_old_prompt_does_not_satisfy_new_acquisition(self):
        self.assertFalse(session_reopened(b'demo> \r\nhost received: bundle\r\n'))
        self.assertFalse(session_reopened(b'demo> bundle\r\n'))

if __name__=='__main__':unittest.main()
