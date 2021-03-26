use neon::{prelude::*, result::Throw};
pub mod codecs;
mod handle_impls;

pub trait IntoHandle {
    type Handle: Value;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> NeonResult<Handle<'c, Self::Handle>>;
}

pub trait FromHandle {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized;
}

pub fn throw<'a, S: AsRef<str>, T>(cx: &mut impl Context<'a>, msg: S) -> Result<T, Throw> {
    let error = cx.error(msg)?;
    cx.throw(error)
}

/// A helper to map Results<T, E> to NeonResult with a string message
pub trait JsMap {
    type Out;
    type Err;
    fn js_map_err<'a, S: AsRef<str>>(
        self,
        cx: &mut impl Context<'a>,
        f: impl FnOnce(Self::Err) -> S,
    ) -> NeonResult<Self::Out>;
}

impl<T, E> JsMap for Result<T, E> {
    type Out = T;
    type Err = E;
    fn js_map_err<'a, S: AsRef<str>>(
        self,
        cx: &mut impl Context<'a>,
        f: impl FnOnce(E) -> S,
    ) -> NeonResult<Self::Out> {
        match self {
            Ok(o) => Ok(o),
            Err(e) => throw(cx, f(e)),
        }
    }
}
/// A helper to conveniently do things like:
/// let v: Duration = fn_ctx.get(0)?;
pub trait Arg<K> {
    fn arg<T: FromHandle>(&mut self, key: K) -> NeonResult<T>;
}

impl Arg<i32> for FunctionContext<'_> {
    fn arg<T: FromHandle>(&mut self, key: i32) -> NeonResult<T> {
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
            js
        }
    }
}
