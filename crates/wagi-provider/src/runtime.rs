use std::hash::{Hash, Hasher};
use std::io::Write;
use std::sync::Arc;

use tempfile::NamedTempFile;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use wasmtime::InterruptHandle;

use kubelet::container::Status;
use kubelet::handle::StopHandler;

use serde::{Deserialize, Serialize};
pub struct Runtime {
    handle: JoinHandle<anyhow::Result<()>>,
    interrupt_handle: InterruptHandle,
}

#[async_trait::async_trait]
impl StopHandler for Runtime {
    async fn stop(&mut self) -> anyhow::Result<()> {
        self.interrupt_handle.interrupt();
        Ok(())
    }

    async fn wait(&mut self) -> anyhow::Result<()> {
        (&mut self.handle).await??;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WagiModulesConfig {
    pub default_host: Option<String>,
    #[serde(rename = "module")]
    pub modules: indexmap::IndexSet<WagiModuleConfig>,
}

impl WagiModulesConfig {
    pub(crate) async fn build_wagi_router(self) -> anyhow::Result<wagi::Router> {
        // TODO: Maybe add wagi::Router::build_from_serde(...)
        let mut temp_file = tempfile::NamedTempFile::new()?;

        {
            // debug
            let config_data = toml::to_string_pretty(&self)?;
            println!("{}", config_data);
        }

        let config_data = toml::to_string(&self)?;
        write!(temp_file, "{}", config_data)?;
        wagi::Router::builder()
            .build_from_modules_toml(temp_file.path())
            .await
    }
}

// Configuration for WAGI modules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WagiModuleConfig {
    pub entrypoint: Option<String>,
    pub route: String,
    pub allowed_hosts: Option<Vec<String>>,
    #[serde(skip_deserializing)]
    pub module: String,
}

impl PartialEq for WagiModuleConfig {
    fn eq(&self, other: &Self) -> bool {
        self.route == other.route
    }
}

impl Eq for WagiModuleConfig {}

impl Hash for WagiModuleConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.route.hash(state);
    }
}

/// Holds our tempfile handle.
pub struct HandleFactory {
    temp: Arc<NamedTempFile>,
}

impl kubelet::log::HandleFactory<tokio::fs::File> for HandleFactory {
    /// Creates `tokio::fs::File` on demand for log reading.
    fn new_handle(&self) -> tokio::fs::File {
        tokio::fs::File::from_std(self.temp.reopen().unwrap())
    }
}

#[tracing::instrument(level = "info", skip(sender, status))]
fn send(sender: &Sender<Status>, name: &str, status: Status) {
    match sender.blocking_send(status) {
        Err(e) => tracing::warn!(error = %e, "error sending wasi status"),
        Ok(_) => tracing::debug!("send completed"),
    }
}
