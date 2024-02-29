import py_scnr
import sys

if len(sys.argv) > 1:
  file = sys.argv[1]
else:
  file = "."

for plist in py_scnr.scan(input = file, filter = ["**/logs/SystemVersion/SystemVersion.plist"], verbose = True):
  json = plist.json()
  print(json)
