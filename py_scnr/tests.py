import unittest
import py_scnr

# test functions have to start with "test"

print(py_scnr.__doc__)

class TestScnr(unittest.TestCase):
  def test_plop(self):
    plop = py_scnr.plop()
    # iter through the list
    for i in plop:
      print(i)
