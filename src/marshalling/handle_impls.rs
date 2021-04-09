use crate::errors::{LazyFmt, MaybeThrown, SafeJsResult, SafeResult};

use super::codecs::*;
use super::*;
use primitive_types::U256;
use rustc_hex::{FromHex as _, ToHex as _};
use secp256k1::{
    recovery::{RecoverableSignature, RecoveryId},
    SecretKey,
};
use std::time::Duration;

impl<T: IntoHandle> IntoHandle for Vec<T> {
    type Handle = JsArray;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        let arr = JsArray::new(cx, 0);
        for i in 0..self.len() {
            let value = self[i].into_handle(cx)?;
            arr.set(cx, i as u32, value)?;
        }
        Ok(arr)
    }
}

impl<T: FromHandle> FromHandle for Option<T> {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        Ok(if handle.is_a::<JsNull>() || handle.is_a::<JsUndefined>() {
            None
        } else {
            Some(T::from_handle(handle, cx)?)
        })
    }
}

impl<T: FromHandle> FromHandle for Vec<T> {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let js_array: JsArray = *handle.downcast().map_err(|e| LazyFmt::new(e))?;
        let len = js_array.len();
        let mut result = Vec::with_capacity(len as usize);
        for i in 0..len {
            let elem = js_array.get(cx, i)?;
            let value = T::from_handle(elem, cx)?;
            result.push(value);
        }
        Ok(result)
    }
}

impl<'a, T0: IntoHandle, T1: IntoHandle> IntoHandle for (T0, T1) {
    type Handle = JsArray;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        let arr = JsArray::new(cx, 0);
        let value = self.0.into_handle(cx)?;
        arr.set(cx, 0, value)?;
        let value = self.1.into_handle(cx)?;
        arr.set(cx, 0, value)?;
        Ok(arr)
    }
}

impl IntoHandle for String {
    type Handle = JsString;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        Ok(JsString::new(cx, self))
    }
}

impl IntoHandle for Vec<u8> {
    // Better would be Uint8Array, but for our use-cases we are turning them
    // into hex strings anyway so we might as well just go straight there.
    type Handle = JsString;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        let hex: String = self.to_hex();
        hex.into_handle(cx)
    }
}

impl<T> IntoHandle for Option<T>
where
    T: IntoHandle,
{
    type Handle = JsValue;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        Ok(match self {
            Some(t) => t.into_handle(cx)?.upcast(),
            None => cx.null().upcast(),
        })
    }
}

impl IntoHandle for U256 {
    type Handle = JsString;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        self.encode().into_handle(cx)
    }
}

impl IntoHandle for f64 {
    type Handle = JsNumber;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        Ok(JsNumber::new(cx, *self))
    }
}

impl IntoHandle for u64 {
    type Handle = JsNumber;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        if *self > 9007199254740991 {
            Err("Number exceeded limits of f64")?
        } else {
            Ok(JsNumber::new(cx, *self as f64))
        }
    }
}

impl IntoHandle for u32 {
    type Handle = JsNumber;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        (*self as f64).into_handle(cx)
    }
}

impl<const N: usize> IntoHandle for [u8; N]
where
    [u8; N]: Encode,
{
    type Handle = JsString;

    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        self.encode().into_handle(cx)
    }
}

impl<const N: usize> FromHandle for [u8; N]
where
    [u8; N]: Decode<str>,
{
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        let a = decode(s.as_str()).map_err(|_| "Failed to parse [u8; N]")?;
        Ok(a)
    }
}

impl FromHandle for String {
    fn from_handle<'a, V: Value>(handle: Handle<V>, _cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        // TODO: (Performance) Eagerly converting to string is not great.
        // This is here because DowncastError is generic over To and From
        // and From is V which would require GAT
        // See also 66e8073c-dd82-4e8e-a62d-0076a1e02f97
        let js_str: JsString = *handle.downcast().map_err(|e| LazyFmt::new(e))?;
        Ok(js_str.value())
    }
}

impl FromHandle for f64 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, _cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let js_num: JsNumber = *handle.downcast().map_err(|e| LazyFmt::new(e))?;
        Ok(js_num.value())
    }
}

impl FromHandle for bool {
    fn from_handle<'a, V: Value>(handle: Handle<V>, _cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let js_bool: JsBoolean = *handle.downcast().map_err(|e| LazyFmt::new(e))?;
        Ok(js_bool.value())
    }
}

impl IntoHandle for bool {
    type Handle = JsBoolean;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        Ok(cx.boolean(*self))
    }
}

impl FromHandle for Vec<u8> {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        let v = s.from_hex().map_err(|_| "Invalid hex")?;
        Ok(v)
    }
}

impl FromHandle for u64 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let number = f64::from_handle(handle, cx)?;

        if number.is_nan() {
            Err("Got NaN for u64")?
        } else if number.is_infinite() {
            Err("Got infinite for u64")?
        } else if number < 0.0 {
            Err("Got negative number for u64")?
        } else if number.fract() != 0.0 {
            Err("Got fractional number for u64")?
        } else if number > 9007199254740991.0 {
            Err("Got number exceeding limits of u64")?
        } else {
            Ok(number as u64)
        }
    }
}

impl FromHandle for Duration {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let ms = f64::from_handle(handle, cx)?;

        if ms.is_nan() {
            Err("Got NaN for Duration")?;
        } else if ms.is_infinite() {
            Err("Got infinite for Duration")?;
        } else if ms < 0.0 {
            Err("Got negative number for Duration")?;
        }

        Ok(Duration::from_secs_f64(ms / 1000.0))
    }
}

impl FromHandle for U256 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        match String::from_handle(handle, cx) {
            Ok(s) => return Ok(decode(&s).map_err(|_| "Failed to parse U256")?),
            // Thrown must never be handled.
            Err(MaybeThrown::Thrown(t)) => return Err(MaybeThrown::Thrown(t)),
            // But unthrown can be ignored since we are going to try u64 next.
            // FIXME: This error will be confusing since this supports cast from
            // either string or u64 but onlyl the error message from the u64
            // branch will be seen.
            _ => {}
        };

        let n = u64::from_handle(handle, cx)?;
        Ok(n.into())
    }
}

impl FromHandle for SecretKey {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        let mut s = s.as_str();
        if s.starts_with("0x") {
            s = &s[2..]
        }
        Ok(s.parse().map_err(|_| "Failed to parse secret key")?)
    }
}

impl FromHandle for RecoverableSignature {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> SafeResult<Self>
    where
        Self: Sized,
    {
        let data = <[u8; 65]>::from_handle(handle, cx)?;

        let recovery_id = data[64];

        let recovery_id = match recovery_id {
            0 | 1 => RecoveryId::from_i32(recovery_id as i32).unwrap(),
            27 | 28 => RecoveryId::from_i32((recovery_id - 27) as i32).unwrap(),
            _ => Err("Invalid recovery id")?,
        };

        Ok(RecoverableSignature::from_compact(&data[..64], recovery_id)
            .map_err(|_| "Failed to parse RecoverableSignature")?)
    }
}

impl IntoHandle for () {
    type Handle = JsUndefined;
    fn into_handle<'c>(&self, cx: &mut impl Context<'c>) -> SafeJsResult<'c, Self::Handle> {
        Ok(cx.undefined())
    }
}
