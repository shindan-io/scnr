import unittest
import py_scnr

# test functions have to start with "test"

print(py_scnr.__doc__)

class TestScnr(unittest.TestCase):
  def test_scan(self):
    for content in py_scnr.scan(input = "src"):
      print(content.content_type())
      print(content)

  def test_jq(self):
    for json in py_scnr.jq(query = ".", input = "src"):
      print(json)
