import py_scnr
import sys

for jq_result in py_scnr.jq( \
    input = sys.argv[1], \
    filter = ["**/logs/Accessibility/TCC.db"], \
    ):
  print(jq_result)
  break

for jq_result in py_scnr.jq( \
    input = sys.argv[1], \
    filter = ["**/logs/itunesstored/downloads.*.sqlitedb*"], \
    verbose = True, \
    ):
  print(jq_result)
  # break
