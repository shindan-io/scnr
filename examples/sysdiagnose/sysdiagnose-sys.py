import py_scnr
import sys

for jq_result in py_scnr.jq( \
    input = sys.argv[1], \
    filter = ["**/logs/SystemVersion/SystemVersion.plist"], \
    query = "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }", \
    ):
  print(jq_result)
