use kubelet::backoff::BackoffStrategy;
use kubelet::pod::state::prelude::*;
use kubelet::state::common::error::Error;
use crate::{PodState, ProviderState};

use super::starting::Starting;

#[derive(Default, Debug, TransitionTo)]
#[transition_to(Starting, Error<crate::WagiProvider>)]
pub struct Initializing;

#[async_trait::async_trait]
impl State<PodState> for Initializing {
    #[tracing::instrument(
        level = "info",
        skip(self, _provider_state, pod_state, _pod),
        fields(pod_name)
    )]
    async fn next(
        self: Box<Self>,
        _provider_state: SharedState<ProviderState>,
        pod_state: &mut PodState,
        _pod: Manifest<Pod>,
    ) -> Transition<PodState> {
        pod_state.crash_loop_backoff_strategy.reset();
        Transition::next(self, Starting)
    }

    async fn status(&self, _pod_state: &mut PodState, _pmeod: &Pod) -> anyhow::Result<PodStatus> {
        Ok(make_status(Phase::Running, "Initializing"))
    }
}
