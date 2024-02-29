import py_scnr
print(py_scnr.__doc__)

for i in py_scnr.scan(input = "../../_samples", filter = ["*.txt"], verbose = True):
  print(i)
