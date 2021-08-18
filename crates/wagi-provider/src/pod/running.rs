use tokio::sync::mpsc::Receiver;

use kubelet::pod::state::prelude::*;
use kubelet::state::common::error::Error;

use super::completed::Completed;
use crate::{PodState, ProviderState};

/// The Kubelet is running the Pod.
#[derive(Debug, TransitionTo)]
#[transition_to(Completed, Error<crate::WagiProvider>)]
pub struct Running {
    rx: Receiver<anyhow::Result<()>>,
}

impl Running {
    pub fn new(rx: Receiver<anyhow::Result<()>>) -> Self {
        Running { rx }
    }
}

#[async_trait::async_trait]
impl State<PodState> for Running {
    async fn next(
        mut self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        _pod_state: &mut PodState,
        pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        let pod = pod.latest();

        while let Some(result) = self.rx.recv().await {
            match result {
                Ok(()) => {
                    return Transition::next(self, Completed);
                }
                Err(e) => {
                    tracing::error!(error = %e);
                    return Transition::Complete(Err(e));
                }
            }
        }
        Transition::next(
            self,
            Error::new(format!(
                "Pod {} container result channel hung up.",
                pod.name()
            )),
        )
    }

    async fn status(&self, _pod_state: &mut PodState, _pod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Running, "Running"))
    }
}
