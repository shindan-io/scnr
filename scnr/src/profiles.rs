use scnr_core::{
  plugins::{json::JsonPlugin, targz::TarGzPlugin, tarxz::TarXzPlugin, text::TextPlugin, zip::ZipPlugin, DefaultPluginPicker},
  ScanError, ScanPlugin,
};
use scnr_plist::PlistPlugin;
use scnr_sqlite::SqlitePlugin;

use crate::options::{CfgProfile, Plugin};

pub fn get_plugin_picker(profile: CfgProfile, cfg: Vec<(String, Plugin)>) -> Result<DefaultPluginPicker, ScanError> {
  let builder = DefaultPluginPicker::builder()
    .push_plugin("*.tar.gz", TarGzPlugin)?
    .push_plugin("*.tar.xz", TarXzPlugin)?
    .push_plugin("*.tgz", TarGzPlugin)?
    .push_plugin("*.zip", ZipPlugin)?
    .push_plugin("*.json", JsonPlugin)?
    .push_plugin("*.txt", TextPlugin)?
    .push_plugin("*.log", TextPlugin)?
    .push_plugin("*.plist", PlistPlugin)?
    .push_plugin("*.db", SqlitePlugin)?
    .push_plugin("*.sqlite", SqlitePlugin)?
    .push_plugin("*.sqlite3", SqlitePlugin)?
    // add more ?
    ;

  let mut builder = match profile {
    CfgProfile::Standard => builder,
    CfgProfile::Sysdiagnose => builder.push_plugin("*.stub", PlistPlugin)?.push_plugin("*.plsql", SqlitePlugin)?,
  };

  for (pattern, plugin) in cfg {
    builder = builder.insert_boxed_plugin(pattern.as_str(), get_plugin(plugin))?;
  }

  builder.build_with_defaults()
}

fn get_plugin(plugin: Plugin) -> Box<dyn ScanPlugin> {
  match plugin {
    Plugin::Json => Box::new(JsonPlugin),
    Plugin::Zip => Box::new(ZipPlugin),
    Plugin::TarGz => Box::new(TarGzPlugin),
    Plugin::TarXz => Box::new(TarXzPlugin),
    Plugin::Text => Box::new(TextPlugin),
    Plugin::Plist => Box::new(PlistPlugin),
    Plugin::Sqlite => Box::new(SqlitePlugin),
  }
}
