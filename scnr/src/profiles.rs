use scnr_core::{
  plugins::{
    bin::BinPlugin, file_system::FileSystemPlugin, ips::IpsPlugin, json::JsonPlugin, targz::TarGzPlugin, tarxz::TarXzPlugin,
    text::TextPlugin, toml::TomlPlugin, xml::XmlPlugin, yaml::YamlPlugin, zip::ZipPlugin, DefaultPluginPicker,
  },
  ScanError, ScanPlugin, ScannerOptions,
};
use scnr_plist::PlistPlugin;
use scnr_sqlite::SqlitePlugin;

use crate::options::{CfgProfile, Plugin};

pub fn get_plugin_picker(
  profile: CfgProfile,
  cfg: &[(String, Plugin)],
  starter: &[Plugin],
  options: &ScannerOptions,
) -> Result<DefaultPluginPicker, ScanError> {
  use scnr_core::plugins::DefaultPluginPickerBuilder;

  fn add_standard_plugins(builder: DefaultPluginPickerBuilder, options: &ScannerOptions) -> Result<DefaultPluginPickerBuilder, ScanError> {
    builder
      .push_plugin("*.tar.gz", TarGzPlugin)?
      .push_plugin("*.tar.xz", TarXzPlugin)?
      .push_plugin("*.tgz", TarGzPlugin)?
      .push_plugin("*.zip", ZipPlugin)?
      .push_plugin("*.json", JsonPlugin)?
      .push_plugin("*.xml", XmlPlugin)?
      .push_plugin("*.yaml", YamlPlugin)?
      .push_plugin("*.yml", YamlPlugin)?
      .push_plugin("*.toml", TomlPlugin)?
      .push_plugin("*.txt", TextPlugin)?
      .push_plugin("*.rs", TextPlugin)?
      .push_plugin("*.log", TextPlugin)?
      .push_plugin("*.csv*", TextPlugin)?
      .push_plugin("*.plist", PlistPlugin)?
      .push_plugin("*.db", SqlitePlugin::new(options))?
      .push_plugin("*.sqlite", SqlitePlugin::new(options))?
      .push_plugin("*.sqlite3", SqlitePlugin::new(options))?
      .push_plugin("*.sqlitedb", SqlitePlugin::new(options))
  }

  let mut builder = match profile {
    CfgProfile::Standard => add_standard_plugins(DefaultPluginPicker::builder(), options)?,
    CfgProfile::Sysdiagnose => add_standard_plugins(DefaultPluginPicker::builder(), options)?
      .push_plugin("*.stub", PlistPlugin)?
      .push_plugin("*.plsql", SqlitePlugin::new(options))?
      .push_plugin("*.epsql", SqlitePlugin::new(options))?
      .push_plugin("*.log*", TextPlugin)?
      .push_plugin("*.ips", IpsPlugin)?,
    CfgProfile::Nothing => DefaultPluginPicker::builder(),
  };

  for (pattern, plugin) in cfg {
    builder = builder.insert_boxed_plugin(pattern.as_str(), get_plugin(*plugin, options))?;
  }

  for plugin in starter {
    builder = builder.push_starter_plugin(get_plugin(*plugin, options))?;
  }

  Ok(match profile {
    CfgProfile::Nothing => builder.build_as_this(),
    _ => builder.build_with_defaults()?,
  })
}

fn get_plugin(plugin: Plugin, options: &ScannerOptions) -> Box<dyn ScanPlugin> {
  match plugin {
    Plugin::FileSystem => Box::new(FileSystemPlugin),
    Plugin::Json => Box::new(JsonPlugin),
    Plugin::Ips => Box::new(IpsPlugin),
    Plugin::Zip => Box::new(ZipPlugin),
    Plugin::TarGz => Box::new(TarGzPlugin),
    Plugin::TarXz => Box::new(TarXzPlugin),
    Plugin::Text => Box::new(TextPlugin),
    Plugin::Plist => Box::new(PlistPlugin),
    Plugin::Sqlite => Box::new(SqlitePlugin::new(options)),
    Plugin::Bin => Box::new(BinPlugin),
  }
}
