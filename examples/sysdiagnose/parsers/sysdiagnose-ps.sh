scnr jq -i $1 -q "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }" -f "**/logs/SystemVersion/SystemVersion.plist"
