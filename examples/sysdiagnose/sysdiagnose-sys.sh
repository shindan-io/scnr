scnr jq \
  -i $1 \
  -f "**/logs/SystemVersion/SystemVersion.plist" \
  -q "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }"
