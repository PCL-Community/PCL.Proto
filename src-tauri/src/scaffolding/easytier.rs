use easytier::{
    common::config::{ConfigFileControl, TomlConfigLoader},
    launcher::NetworkInstance,
};

pub struct EasyTierHolder {
    instance: NetworkInstance,
    runtime: tokio::runtime::Runtime,
}

impl EasyTierHolder {
    fn create(config: TomlConfigLoader) -> anyhow::Result<Self> {
        let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
        instance.start()?;
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        
        Ok(Self { instance, runtime })
    }
}
