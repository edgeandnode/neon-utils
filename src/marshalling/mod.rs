use neon::prelude::*;
pub mod codecs;
mod handle_impls;
use crate::errors::{SafeJsResult, SafeResult};

pub trait IntoHandle {
    type Handle: Value;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle>;
}

pub trait FromHandle {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized;
}

/// A helper to conveniently do things like:
/// let v: Duration = fn_ctx.get(0)?;
pub trait Arg<K> {
    fn arg<T: FromHandle>(&mut self, key: K) -> SafeResult<T>;
}

impl<O: neon::object::This> Arg<i32> for CallContext<'_, O> {
    fn arg<T: FromHandle>(&mut self, key: i32) -> SafeResult<T> {
        let arg = self.argument::<JsValue>(key)?;
        T::from_handle(arg, self)
    }
}

#[macro_export]
macro_rules! js_object {
    ($cx:expr => {$($k:ident: $v:expr,)*}) => {
        {
            let js = JsObject::new($cx);
            $(
                let handle = ($v).into_handle($cx)?;
                js.set($cx, stringify!($k), handle)?;
            )*
            Ok(js)
        }
    }
}
