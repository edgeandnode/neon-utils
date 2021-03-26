use crate::marshalling::{IntoError, IntoHandle};
use atomic_take::AtomicTake;
use neon::prelude::*;

struct TaskWrapper<F> {
    f: AtomicTake<F>,
}

impl<F> TaskWrapper<F> {
    pub fn new(f: F) -> Self {
        Self {
            f: AtomicTake::new(f),
        }
    }
}

impl<F, Ok, Err> Task for TaskWrapper<F>
where
    F: 'static + Send + FnOnce() -> Result<Ok, Err>,
    Err: 'static + Send + IntoError,
    Ok: 'static + Send + IntoHandle,
{
    type Output = Ok;
    type Error = Err;
    type JsEvent = <Ok as IntoHandle>::Handle;

    fn perform(&self) -> Result<Self::Output, Self::Error> {
        let f = self.f.take().unwrap();
        f()
    }

    fn complete(
        self,
        mut cx: TaskContext,
        result: Result<Self::Output, Self::Error>,
    ) -> JsResult<Self::JsEvent> {
        result.into_handle(&mut cx)
    }
}

/// Runs a function asynchronously then calls
/// the callback with the result.
pub fn run_async<'c, F, Ok, Err>(callback: Handle<JsFunction>, f: F)
where
    F: 'static + Send + FnOnce() -> Result<Ok, Err>,
    Err: 'static + Send + IntoError,
    Ok: 'static + Send + IntoHandle,
{
    let task = TaskWrapper::new(f);
    task.schedule(callback);
}
