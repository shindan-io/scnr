## Demo/Presentation example codes 

#### First attemps :
```rust
impl DeserialiseParser for ListOfScannedNetworksWithPrivateMacParser {
  type DeserializedType = VecListOfScannedNetworksWithPrivateMac;

  fn deserialize_reader(&self, reader: impl Read + Seek + 'static) -> Result<Self::DeserializedType, ParseError> {
    Ok(plist::from_reader(reader)?)
  }
```
```rust
#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VecListOfScannedNetworksWithPrivateMac {
  #[serde(rename = "List of scanned networks with private mac")]
  pub list_of_scanned_networks_with_private_mac: Vec<ListOfScannedNetworksWithPrivateMac>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOfScannedNetworksWithPrivateMac {
  #[serde(rename = "MacGenerationTimeStamp")]
  pub mac_generation_time_stamp: Option<String>,
  #[serde(rename = "PrivateMacFutureMacAddress")]
  pub private_mac_future_mac_address: Option<plist::Data>,
  #[serde(rename = "BlockRotation")]
  pub block_rotation: Option<bool>,
  // ...
  // 50 MORE LINES
  // ...
  #[serde(rename = "FailureCountThresholdCurrent")]
  pub failure_count_threshold_current: Option<i64>,
  #[serde(rename = "NetworkWasCaptive")]
  pub network_was_captive: Option<bool>,
}
```

#### New ways
```rust
crate::parse::scnr::impl_scnr_parser_json!(
  ListOfScannedNetworksWithPrivateMacParser,
  "**/WiFi/com.apple.wifi-private-mac-networks.plist",
  |json, root_path, rel_path| {
    let objs = jq(
      json,
      r#"
        ."List of scanned networks with private mac"[] 
          | select( type == "object" ) 
          | select( .lastJoined != null )
          | { "addedAt": .lastJoined, "open": .IsOpenNetwork, "ssid":.SSID_STR }
      "#,
    )?;

    for obj in objs {
      // .. do something with JSON values
    }
  }
);

```

#### Choose your flavor - command line
```sh
scnr jq \
  -i $1 \
  -f "**/logs/SystemVersion/SystemVersion.plist" \
  -q "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }"
```

#### Choose your flavor - python
```python
import py_scnr
import sys

for jq_result in py_scnr.jq( \
    input = sys.argv[1], \
    filter = ["**/logs/SystemVersion/SystemVersion.plist"], \
    query = "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }", \
    ):
  print(jq_result)
```

#### Archive transparency
```sh
$ scnr jq \
 -i sysdiagnose_2023.10.26_14-40-37+0200_iPhone-OS_iPhone_19H349 \
 -f "**/logs/SystemVersion/SystemVersion.plist" \
 -q "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }"
```
```sh
$ scnr jq \
 -i sysdiagnose_2023.10.26_14-40-37+0200_iPhone-OS_iPhone_19H349.tar.gz \
 -f "**/logs/SystemVersion/SystemVersion.plist" \
 -q "{ ProductName, ProductVersion, ProductBuildVersion, BuildID, SystemImageID }"
```
##### Same result
```json
{
  "ProductName": "iPhone OS",
  "ProductVersion": "15.7.6",
  "ProductBuildVersion": "19H349",
  "BuildID": "F66FFDFE-E5A9-11ED-B408-720BCFA60583",
  "SystemImageID": "5FAC5A2B-DB57-4EDD-A576-4C662CD5B428"
}
```

#### Grep through sqlite ? in an archive ?
```sh
scnr scan -i _samples -f *w.tar.gz/*.db | grep -B 2 -A 2 Islands
```
```txt
  {
    "country_id": 32,
    "country": "Faroe Islands",
    "last_update": "2020-12-23 07:12:13"
  },
--
  {
    "country_id": 106,
    "country": "Virgin Islands, U.S.",
    "last_update": "2020-12-23 07:12:14"
  },
```

#### Extract command
```sh
scnr extract -i sysdiagnose_*_20I444.tar.gz -o sysdiag_expanded -p sysdiagnose
more sysdiag_expanded/...../logs/Accessibility/TCC.db/access
```
```json
[
  {
	"service": "kTCCServiceMotion",
	"client": "com.apple.Health",
	"client_type": 0,
	"auth_value": 2,
	"auth_reason": 4,
	"auth_version": 1,
	"csreq": null,
	"policy_id": null,
	"indirect_object_identifier_type": 0,
	"indirect_object_identifier": "UNUSED",
	"indirect_object_code_identity": null,
	"flags": 0,
	"last_modified": 1684007050
  },
  // ... ...
]
```

#### 
