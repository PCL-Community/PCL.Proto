// use easytier::{
//     common::config::{ConfigFileControl, TomlConfigLoader},
//     launcher::NetworkInstance,
//     rpc_service::InstanceRpcService,
// };
// use std::sync::Arc;

// pub struct EasyTierHolder {
//     instance: NetworkInstance,
// }

// impl EasyTierHolder {
//     fn create(config: TomlConfigLoader) -> anyhow::Result<Self> {
//         let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
//         instance.start()?;

//         // 简单等待实例启动
//         std::thread::sleep(std::time::Duration::from_secs(2));

//         // 检查启动状态
//         if !instance.is_easytier_running() {
//             if let Some(error) = instance.get_latest_error_msg() {
//                 return Err(anyhow::anyhow!("Failed to start EasyTier: {}", error));
//             }
//             return Err(anyhow::anyhow!("Failed to start EasyTier"));
//         }

//         Ok(Self { instance })
//     }

//     // 获取 API 服务
//     fn get_api_service(&self) -> Option<Arc<dyn InstanceRpcService>> {
//         self.instance.get_api_service()
//     }

//     // 检查运行状态
//     fn is_running(&self) -> bool {
//         self.instance.is_easytier_running()
//     }
// }
