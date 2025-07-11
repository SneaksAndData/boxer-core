use anyhow::bail;
use async_trait::async_trait;
use kube::Config;
use kube::config::Kubeconfig;
use log::{debug, info};
use serde_yml::from_str;
use std::process::Command;
use std::sync::Arc;

pub fn from_command() -> Arc<dyn KubeConfigLoader<ConfigSource = String>> {
    Arc::new(ExecutableKubeConfigLoader)
}

pub fn from_file() -> Arc<dyn KubeConfigLoader<ConfigSource = String>> {
    Arc::new(FileKubeConfigLoader)
}

#[async_trait]
pub trait KubeConfigLoader: Send + Sync {
    type ConfigSource;
    async fn load(&self, source: &Self::ConfigSource) -> anyhow::Result<Config>;
}

struct ExecutableKubeConfigLoader;

#[async_trait]
impl KubeConfigLoader for ExecutableKubeConfigLoader {
    type ConfigSource = String;

    async fn load(&self, source: &Self::ConfigSource) -> anyhow::Result<Config> {
        info!("Configuring Kubernetes backend with command: {:?}", source);
        let output = Command::new("sh").arg("-c").arg(source).output()?;
        if !output.status.success() {
            bail!(
                "Failed to execute command: {:?}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let kubeconfig_string = String::from_utf8(output.stdout)?;
        debug!("Kubeconfig used by the backend:\n{:?}", kubeconfig_string);
        let kubeconfig: Kubeconfig = from_str(&kubeconfig_string)?;
        Ok(Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?)
    }
}

struct FileKubeConfigLoader;

#[async_trait]
impl KubeConfigLoader for FileKubeConfigLoader {
    type ConfigSource = String;

    async fn load(&self, source: &Self::ConfigSource) -> anyhow::Result<Config> {
        info!("Configuring Kubernetes backend with kubeconfig file: {:?}", source);
        let kubeconfig_string = std::fs::read_to_string(source)?;
        debug!("Kubeconfig used by the backend:\n{:?}", kubeconfig_string);
        let kubeconfig: Kubeconfig = from_str(&kubeconfig_string)?;
        Ok(Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?)
    }
}
