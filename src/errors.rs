use crate::marshalling::*;
use neon::{prelude::*, result::Throw};

impl<Ok, Err> IntoHandle for Result<Ok, Err>
where
    Ok: IntoHandle,
    Err: IntoError,
{
    type Handle = <Ok as IntoHandle>::Handle;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, Self::Handle> {
        match self {
            Ok(ok) => ok.into_handle(cx),
            Err(e) => {
                let e = e.into_error(cx)?;
                cx.throw(e)
            }
        }
    }
}

pub fn throw<'a, S: AsRef<str>, T>(cx: &mut impl Context<'a>, msg: S) -> Result<T, Throw> {
    let error = cx.error(msg)?;
    cx.throw(error)
}
