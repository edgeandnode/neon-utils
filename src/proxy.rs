use neon::{prelude::*, result::NeonResult};
use std::sync::Arc;

use crate::errors::{IntoError, MaybeThrown};

/// Provides a way to easily share data across
/// threads when wrapped in a JavaScript class
pub struct Proxy<T>(Arc<T>);

impl<T> Clone for Proxy<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Proxy<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> From<T> for Proxy<T> {
    fn from(value: T) -> Proxy<T> {
        Proxy::<T>::new(value)
    }
}

// This was supposed to be part of Terminal, but can't do it without GAT.
pub trait ProxyTerminal: Sized {
    type Out;
    fn finish<'c>(self, cx: impl Context<'c>) -> NeonResult<Self::Out>;
}

impl<T> ProxyTerminal for Result<Proxy<T>, MaybeThrown> {
    type Out = Proxy<T>;
    fn finish<'c>(self, mut cx: impl Context<'c>) -> NeonResult<Proxy<T>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(e) => match e {
                MaybeThrown::Thrown(t) => Err(t),
                MaybeThrown::Unthrown(e) => match e.into_error(&mut cx) {
                    Ok(e) => cx.throw(e),
                    Err(e) => Err(e),
                },
            },
        }
    }
}
