use std::time::Duration;

use crate::p2p::connection::P2pConnectionState;
use crate::p2p::P2pPeerStatus;

use super::*;

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingRandomInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingInitAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingReconnectAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        if !self.is_enabled(&state.p2p) {
            return false;
        }

        let Some(peer) = state.p2p.peers.get(self.opts.peer_id()) else {
            return false;
        };
        let delay_passed = match &peer.status {
            P2pPeerStatus::Connecting(P2pConnectionState::Outgoing(
                P2pConnectionOutgoingState::Error { time, .. },
            ))
            | P2pPeerStatus::Disconnected { time, .. } => {
                state.time().checked_sub(*time) >= Some(Duration::from_secs(30))
            }
            _ => true,
        };
        delay_passed
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingOfferSdpCreatePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingOfferSdpCreateErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingOfferSdpCreateSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingOfferReadyAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingOfferSendSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingAnswerRecvPendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingAnswerRecvErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingAnswerRecvSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingFinalizePendingAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingFinalizeErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingFinalizeSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingTimeoutAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        let peer = state.p2p.peers.get(&self.peer_id);
        let timed_out = peer
            .and_then(|peer| peer.status.as_connecting()?.as_outgoing())
            .map_or(false, |s| s.is_timed_out(state.time()));
        timed_out && self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingErrorAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}

impl redux::EnablingCondition<crate::State> for P2pConnectionOutgoingSuccessAction {
    fn is_enabled(&self, state: &crate::State) -> bool {
        self.is_enabled(&state.p2p)
    }
}
