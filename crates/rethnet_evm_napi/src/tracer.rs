mod js_tracer;

use napi_derive::napi;
use rethnet_evm::{state::StateError, AsyncDatabase, Inspector};

pub use self::js_tracer::{TracingMessage, TracingMessageResult, TracingStep};

#[napi]
pub struct Tracer {
    inner: Box<JsTracer>,
}

impl Tracer {
    pub fn as_dyn_inspector(
        &self,
    ) -> Box<(dyn Inspector<AsyncDatabase<napi::Error, StateError>> + Send + 'static)> {
        self.inner.clone()
    }
}

#[napi]
impl Tracer {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Box::new(JsTracer::default()),
        }
    }
}
