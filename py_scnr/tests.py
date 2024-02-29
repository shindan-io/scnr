import unittest
import py_scnr

# test functions have to start with "test"

print(py_scnr.__doc__)

class TestScnr(unittest.TestCase):
  def test_scan(self):
    for i in py_scnr.scan(input = ".venv/lib/python3.10/site-packages/py_scnr-0.1.0.dist-info/"):
      print(i)

  def test_jq(self):
    for i in py_scnr.jq(".", input = ".venv/lib/python3.10/site-packages/py_scnr-0.1.0.dist-info/"):
      print(i)
