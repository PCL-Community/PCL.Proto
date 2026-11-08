#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    #[default]
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    LegacyFabric,
    Quilt,
    Optifine,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FabricFamily {
    Fabric,
    Legacyfabric,
}
