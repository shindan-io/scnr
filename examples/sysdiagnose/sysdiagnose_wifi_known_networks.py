import py_scnr
import sys

for jq_result in py_scnr.jq( \
    input = sys.argv[1], \
    filter = ["**/com.apple.wifi.known-networks.plist"], \
    query = ".", \
    ):
  print(jq_result)
