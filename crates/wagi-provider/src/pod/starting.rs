use std::net::SocketAddr;

use tracing::{info, instrument};

use kubelet::pod::state::prelude::*;
use crate::{PodState, ProviderState};

use kubelet::state::common::error::Error;
use super::running::Running;

use crate::pod::{WAGI_ANNOTATION_KEY_DEFAULT_HOST, WAGI_ANNOTATION_KEY_MODULES};
use crate::runtime::{WagiModuleConfig, WagiModulesConfig};

#[derive(Default, Debug, TransitionTo)]
#[transition_to(Running, Error<crate::WagiProvider>)]
/// The Kubelet is starting the Pod containers
pub(crate) struct Starting;

#[async_trait::async_trait]
impl State<PodState> for Starting {
    #[instrument(
        level = "info",
        skip(self, _provider_state, _pod_state, pod),
        fields(pod_name)
    )]
    async fn next(
        self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        _pod_state: &mut PodState,
        pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        let pod = pod.latest();

        tracing::Span::current().record("pod_name", &pod.name());
        
        info!("Starting WAGI pod");

        let mut wagi_modules_config = WagiModulesConfig::default();

        if let Some(default_host_annotation) = pod.annotations().get(WAGI_ANNOTATION_KEY_DEFAULT_HOST) {
            wagi_modules_config.default_host = Some(default_host_annotation.clone());
        }

        if let Some(modules_annotation) = pod.annotations().get(WAGI_ANNOTATION_KEY_MODULES) {
            match serde_json::from_str(&modules_annotation) {
                Ok(modules) => {
                    let modules_map: indexmap::IndexMap<String, WagiModuleConfig> = pod
                        .clone()
                        .containers()
                        .into_iter()
                        .filter_map(|c| c
                            .image()
                            .ok()
                            .flatten()
                            .map(|image| (c.name().to_owned(), image.to_string())))
                            .fold(modules, |mut modules, (name, image)| {
                                modules.entry(name).and_modify(|m| m.module = format!("oci://{}", image));
                                modules
                            });

                        wagi_modules_config.modules = modules_map
                            .into_iter()
                            .map(|(_, m)| m)
                            .collect();
                },
                Err(parse_err) => {
                    return Transition::next(self, Error::new(format!(
                        "Error parsing WAGI annotation for key {:?}: {}",
                        WAGI_ANNOTATION_KEY_MODULES,
                        parse_err,
                    )));
                }
            }
        }

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tokio::task::spawn(async move {
            let default_host = wagi_modules_config
                .default_host
                .clone()
                .unwrap_or("127.0.0.1:3000".to_string()).parse::<SocketAddr>()
                .map_err(|e| anyhow::anyhow!("{}", e));

            let server_addr = match default_host {
                Ok(addr) => addr,
                Err(error) => {
                    tracing::error!(error = %error);
                    return tx.send(Err(error)).await;
                },
            };

            match wagi_modules_config.build_wagi_router().await {
                Ok(router) => {
                    use hyper::service::{make_service_fn, service_fn};
                    use hyper::server::conn::AddrStream;
                    use hyper::Server;

                    let mk_svc = make_service_fn(move |conn: &AddrStream| {
                        let addr = conn.remote_addr();
                        let r = router.clone();
                        async move {
                            Ok::<_, std::convert::Infallible>(service_fn(move |req| {
                                let r2 = r.clone();
                                async move { r2.route(req, addr).await }
                            }))
                        }
                    });

                    let result = Server::bind(&server_addr)
                        .serve(mk_svc)
                        .await
                        .map_err(|e| anyhow::anyhow!("{}", e));

                    info!("WAGI pod started");
                    tx.send(result).await
                },
                Err(error) => {
                    tx.send(Err(anyhow::anyhow!("{}", error))).await
                },
            }
        });

        Transition::next(self, Running::new(rx))
    }

    async fn status(&self, _pod_state: &mut PodState, _pod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Pending, "Starting"))
    }
}