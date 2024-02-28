import unittest
import py_scnr

# test functions have to start with "test"

print(py_scnr.__doc__)

class TestScnr(unittest.TestCase):
  def test_scan(self):
    for i in py_scnr.scan():
      print(i)
