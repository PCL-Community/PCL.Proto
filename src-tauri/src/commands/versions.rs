use crate::{
    io::net::http::api_client::{self, plugins::McPluginReport},
    minecraft::mod_loader,
};
use tauri::State;

#[tauri::command]
pub async fn get_fabric_installers(
    client: State<'_, &api_client::MinecraftApiClient>,
    family: api_client::FabricFamily,
) -> Result<Vec<McPluginReport>, String> {
    client
        .get_fabric_installers(family)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_version_manifest(
    client: State<'_, &api_client::MinecraftApiClient>,
) -> Result<api_client::game::McVersionManifest, String> {
    match client
        .get_with_endpoint::<api_client::game::McVersionManifest>(
            api_client::game::VERSION_MANIFEST_ENDPOINT,
            true,
        )
        .await
    {
        Ok(manifest) => Ok(manifest),
        Err(e) => Err(format!("get_version_manifest: {:?}", e)),
    }
}

/// return the version json
#[tauri::command]
pub async fn handle_clicked_on_version(
    client: State<'_, &api_client::MinecraftApiClient>,
    id: &str,
) -> Result<api_client::game::McVersion, String> {
    let temp_dir = std::env::temp_dir().join(format!("pcl-proto-{}", id));
    log::debug!("selected tmp: {:?}", temp_dir);
    let version_detail = client
        .get_version_details(id, &temp_dir)
        .await
        .map_err(|err| err.to_string())?;
    Ok(version_detail)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_plugin_versions(
    api_client: State<'_, &api_client::MinecraftApiClient>,
    plugin_type: mod_loader::ModLoaderType,
    mc_version: &str,
) -> Result<Vec<McPluginReport>, String> {
    let verisons = match plugin_type {
        mod_loader::ModLoaderType::Forge => api_client.get_forge_versions(mc_version).await,
        mod_loader::ModLoaderType::NeoForge => api_client.get_neoforge_versions(mc_version).await,
        mod_loader::ModLoaderType::Fabric => {
            api_client
                .get_fabric_versions(mc_version, api_client::FabricFamily::Fabric)
                .await
        }
        mod_loader::ModLoaderType::LegacyFabric => {
            api_client
                .get_fabric_versions(mc_version, api_client::FabricFamily::Legacyfabric)
                .await
        }
        _ => Err(api_client::McApiError::PluginMismatch(plugin_type)),
    };
    verisons.map_err(|err| err.to_string())
}
