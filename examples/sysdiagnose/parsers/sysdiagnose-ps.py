import py_scnr
import sys

for jq_result in py_scnr.jq( \
    "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }", \
    input = sys.argv[1], \
    filter = ["**/logs/SystemVersion/SystemVersion.plist"], \
    ):
  print(jq_result)
