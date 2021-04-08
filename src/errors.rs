use crate::marshalling::*;
use neon::{
    handle::{DowncastError, Managed},
    prelude::*,
    result::Throw,
};
use never::Never;
use std::fmt;

/// This type is to handle a problem that neon creates.
/// First, if you return Err(Throw) from a function without calling throw() neon will segfault.
/// Second, if you handle an Result::Err(Throw) returned by throw() then neon will still throw an Error later.
/// Those make handling errors kind of tricky. If functions return JsResult for example you have to know
/// which Throws are segfaulting and which ones are unhandleable at the callee. If you make them all proper throw()
/// that cannot segfault then no errors can be handled. If instead you never return Throw and opt for some kind of
/// IntoError then you can't interact with any of neon's functions which do return JsResult.
/// If that wasn't enough, IntoError naturally requires a generic Context making it not play well with
/// the ? operator and not object safe. Fun!
/// This is the least bad thing I came up with.
///
/// TODO: I can't for the life of me make a foolproof throw API right now.
/// What we want:
///   * Whenever Err(Throw) is encountered it always propagates up and
///     basically can not be handled without going out of one's way.
///   * Err(Unthrown) can be handled, but if unhandled should eventually always be Thrown
///   * Ok to proceed normally
///   * Readable.
///
/// Example problem - with this code I wanted to write..
///   if let Ok(s) = String::from_handle(...
/// But that innocuous looking code suppresses Err(Thrown).
/// For today I will be content with the latest upgrade having MaybeThrown
/// which at least makes it possible to write correct code, even if it's
/// error prone. Before MaybeThrown no errors could be handled ever.
/// There's still nothing stopping you from instantiating a Throw and segfaulting.
pub type SafeResult<Ok> = Result<Ok, MaybeThrown>;
pub type SafeJsResult<'c, Ok> = SafeResult<Handle<'c, Ok>>;

pub trait IntoError {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError>;
}

impl IntoError for Never {
    fn into_error<'c>(&self, _cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        unreachable!()
    }
}

pub enum MaybeThrown {
    Thrown(Throw),
    Unthrown(SafeErr),
}

impl From<Throw> for MaybeThrown {
    fn from(throw: Throw) -> Self {
        MaybeThrown::Thrown(throw)
    }
}

impl<T> From<T> for MaybeThrown
where
    T: Into<SafeErr>,
{
    fn from(t: T) -> Self {
        MaybeThrown::Unthrown(t.into())
    }
}

impl<T: Value, F: Value> IntoError for DowncastError<T, F> {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        let msg = format!("{}", self);
        cx.error(msg)
    }
}

impl IntoError for String {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        cx.error(self.as_str())
    }
}

impl IntoError for &'_ str {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        cx.error(self)
    }
}

pub trait Terminal {
    type Handle: Value;
    // This takes Context by value to prevent interacting with JS
    // after possibly throwing. The reasoning here is that with neon
    // if you return Err without throwing it segfaults, but if you
    // throw then handle and return Ok it will still throw. So,
    // this API gives a canonical place to do the actual throw.
    fn finish<'c>(self, cx: impl Context<'c>) -> JsResult<'c, Self::Handle>;
}

impl MaybeThrown {
    pub fn finish<'c, Any: Managed>(self, mut cx: impl Context<'c>) -> JsResult<'c, Any> {
        match self {
            MaybeThrown::Thrown(t) => Err(t),
            MaybeThrown::Unthrown(e) => match e.into_error(&mut cx) {
                Ok(ok) => cx.throw(ok),
                Err(err) => Err(err),
            },
        }
    }
}

impl<Ok> Terminal for Result<Ok, MaybeThrown>
where
    Ok: IntoHandle,
{
    type Handle = <Ok as IntoHandle>::Handle;
    fn finish<'c>(self, mut cx: impl Context<'c>) -> JsResult<'c, Self::Handle> {
        match self {
            Ok(ok) => match ok.into_handle(&mut cx) {
                Ok(ok) => Ok(ok),
                Err(e) => e.finish(cx),
            },
            Err(e) => e.finish(cx),
        }
    }
}

impl<Ok, Err> Terminal for Result<Ok, Err>
where
    Ok: IntoHandle,
    Err: IntoError,
{
    type Handle = <Ok as IntoHandle>::Handle;
    fn finish<'c>(self, mut cx: impl Context<'c>) -> JsResult<'c, Self::Handle> {
        match self {
            Ok(ok) => match ok.into_handle(&mut cx) {
                Ok(ok) => Ok(ok),
                Err(e) => e.finish(cx),
            },
            Err(e) => {
                let e = e.into_error(&mut cx)?;
                cx.throw(e)
            }
        }
    }
}

// After much consternation, I am enumerating all errors instead of using generics.
// It's a fair amount of work to get this module to compile with generic errors,
// but it works. Then you run into an issue on the usage end where calls to .arg
// require layers of error handling to coerce errors to the same type. In this module
// using an Either<A, B> struct was sufficient, but when you call .arg 10 times things
// start to get really hairy for the consuming module.
pub enum SafeErr {
    StaticStr(&'static str),
    String(String),
    LazyFmt(LazyFmt),
}

impl From<&'static str> for SafeErr {
    fn from(v: &'static str) -> Self {
        Self::StaticStr(v)
    }
}
impl From<String> for SafeErr {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
impl From<LazyFmt> for SafeErr {
    fn from(v: LazyFmt) -> Self {
        Self::LazyFmt(v)
    }
}

impl IntoError for SafeErr {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        match self {
            SafeErr::StaticStr(s) => s.into_error(cx),
            SafeErr::String(s) => s.into_error(cx),
            SafeErr::LazyFmt(l) => l.into_error(cx),
        }
    }
}

pub struct LazyFmt(Box<dyn fmt::Display>);

impl LazyFmt {
    pub fn new<T>(value: T) -> Self
    where
        T: 'static + fmt::Display,
    {
        Self(Box::new(value))
    }
}

impl fmt::Display for LazyFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl IntoError for LazyFmt {
    fn into_error<'c>(&self, cx: &mut impl Context<'c>) -> JsResult<'c, JsError> {
        let s = format!("{}", self);
        cx.error(s)
    }
}
