use k8s_openapi::api::core::v1::Namespace;
use kube::api::PostParams;
use kube::config::Kubeconfig;
use kube::{Api, Client, Config};
use log::info;
use serde_json::json;
use std::process::Command;
use uuid::Uuid;

//#[cfg(feature = "testing")]
pub mod api_client_context;
pub mod api_extensions;
pub mod spin_lock_kubernetes_resource_manager_context;
pub mod temp_namespace_context;

/// COVERAGE: disabled since this is a testing helper
#[cfg_attr(coverage, coverage(off))]
pub async fn get_kubeconfig() -> anyhow::Result<Config> {
    let output = Command::new("kind")
        .args(&["get", "kubeconfig", "--name", "kind"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to get kubeconfig: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let kubeconfig_string = String::from_utf8(output.stdout)?;
    info!("Kubeconfig used by the tests:\n{}", kubeconfig_string);
    let kubeconfig: Kubeconfig = serde_norway::from_str(&kubeconfig_string)?;
    let config = Config::from_custom_kubeconfig(kubeconfig, &Default::default()).await?;
    Ok(config)
}

pub async fn create_namespace() -> anyhow::Result<String> {
    let config = get_kubeconfig().await?;
    let client = Client::try_from(config.clone())?;

    let namespace_name = Uuid::new_v4().to_string();
    info!("Using namespace: {}", namespace_name);

    let namespaces: Api<Namespace> = Api::all(client);
    let namespace_definition = json!({
        "metadata": {
            "name": namespace_name
        }
    });

    let ns = serde_json::from_value(namespace_definition)?;
    namespaces
        .create(&PostParams::default(), &ns)
        .await
        .expect("Create Namespace failed");
    info!("Namespace {} created successfully", namespace_name);
    Ok(namespace_name)
}
