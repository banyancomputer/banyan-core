#![allow(dead_code)]

mod helpers;

pub(crate) mod prelude {
    #![allow(unused_imports)]

    pub(crate) use crate::tests::MockState;
    pub(crate) use crate::tests::helpers::TestClient;
}

#[derive(Clone)]
pub(crate) struct MockState;
