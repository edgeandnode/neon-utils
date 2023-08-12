use crate::errors::{IntoError, Terminal};
use crate::marshalling::IntoHandle;
use atomic_take::AtomicTake;
use neon::prelude::*;
use std::sync::Arc;

/// Runs a function asynchronously then calls
/// the callback with the result.
pub fn run_async<'c, F, Ok, Err>(mut cx: FunctionContext, callback: Handle<JsFunction>, f: F)
where
    F: 'static + Send + FnOnce() -> Result<Ok, Err>,
    Err: 'static + Send + IntoError,
    Ok: 'static + Send + IntoHandle,
    Result<Ok, Err>: Terminal<Handle = Ok::Handle>,
{
    let f_taken = Arc::new(AtomicTake::new(f));
    let channel = cx.channel();
    let callback = callback.root(&mut cx);
    std::thread::spawn(move || {
        let f_unwrapped = f_taken.take().unwrap();
        let result = f_unwrapped().map_err(|err| err.into_error(&mut cx));
        let cb_args = match result {
            Ok(ok) => vec![cx.null().upcast::<JsValue>()],
            Err(err) => vec![cx.string(err.to_string()).upcast::<JsValue>()],
        };
        channel.send(move |mut _cx| {
            let callback = callback.into_inner(&mut _cx);
            let _r: Handle<JsValue> = callback.call_with(&mut _cx).args(cb_args).apply(&mut _cx)?;
            Ok(())
        });
    });
}
