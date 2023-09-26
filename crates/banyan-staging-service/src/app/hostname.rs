use url::Url;

use crate::app::State;

pub struct Hostname(pub Url);

impl axum::extract::FromRef<State> for Hostname {
    fn from_ref(state: &State) -> Self {
        Hostname(state.hostname())
    }
}
