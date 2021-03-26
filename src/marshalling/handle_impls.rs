use super::codecs::*;
use super::*;
use crate::prelude::*;
use primitive_types::U256;
use rustc_hex::FromHex as _;
use rustc_hex::ToHex as _;
use secp256k1::SecretKey;
use std::time::Duration;

impl<'h, T: Value> IntoHandle<'h> for Handle<'h, T> {
    type Handle = T;
    fn into_handle(&self, _cx: &mut impl Context<'h>) -> NeonResult<Handle<'h, Self::Handle>> {
        Ok(*self)
    }
}

impl<'h, T: IntoHandle<'h>> IntoHandle<'h> for Vec<T> {
    type Handle = JsArray;
    fn into_handle(&self, cx: &mut impl Context<'h>) -> NeonResult<Handle<'h, Self::Handle>> {
        let arr = JsArray::new(cx, 0);
        for i in 0..self.len() {
            let value = self[i].into_handle(cx)?;
            arr.set(cx, i as u32, value)?;
        }
        Ok(arr)
    }
}

impl<T: FromHandle> FromHandle for Option<T> {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
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
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let js_array: JsArray = *handle.downcast_or_throw(cx)?;
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

impl<'a, T0: IntoHandle<'a>, T1: IntoHandle<'a>> IntoHandle<'a> for (T0, T1) {
    type Handle = JsArray;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        let arr = JsArray::new(cx, 0);
        let value = self.0.into_handle(cx)?;
        arr.set(cx, 0, value)?;
        let value = self.1.into_handle(cx)?;
        arr.set(cx, 0, value)?;
        Ok(arr)
    }
}

impl<'a> IntoHandle<'a> for String {
    type Handle = JsString;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        Ok(JsString::new(cx, self))
    }
}

impl<'a> IntoHandle<'a> for Vec<u8> {
    // Better would be Uint8Array, but for our use-cases we are turning them
    // into hex strings anyway so we might as well just go straight there.
    type Handle = JsString;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        let hex: String = self.to_hex();
        hex.into_handle(cx)
    }
}

impl<'a, T> IntoHandle<'a> for Option<T>
where
    T: IntoHandle<'a>,
{
    type Handle = JsValue;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        Ok(match self {
            Some(t) => t.into_handle(cx)?.upcast(),
            None => cx.null().upcast(),
        })
    }
}

impl<'a> IntoHandle<'a> for U256 {
    type Handle = JsString;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        self.encode().into_handle(cx)
    }
}

impl<'a> IntoHandle<'a> for f64 {
    type Handle = JsNumber;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        Ok(JsNumber::new(cx, *self))
    }
}

impl<'a> IntoHandle<'a> for u64 {
    type Handle = JsNumber;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        if *self > 9007199254740991 {
            throw(cx, "Number exceeded limits of f64")
        } else {
            (*self as f64).into_handle(cx)
        }
    }
}

impl<'a> IntoHandle<'a> for Bytes32 {
    type Handle = JsString;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        self.encode().into_handle(cx)
    }
}

impl<'a> IntoHandle<'a> for Address {
    type Handle = JsString;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        self.encode().into_handle(cx)
    }
}

impl FromHandle for String {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let js_str: JsString = *handle.downcast_or_throw(cx)?;
        Ok(js_str.value())
    }
}

impl FromHandle for Address {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        decode(s.as_str()).js_map_err(cx, |_| format!("Failed to parse Address from \"{}\"", s))
    }
}

impl FromHandle for f64 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let js_num: JsNumber = *handle.downcast_or_throw(cx)?;
        Ok(js_num.value())
    }
}

impl FromHandle for bool {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let js_bool: JsBoolean = *handle.downcast_or_throw(cx)?;
        Ok(js_bool.value())
    }
}

impl<'a> IntoHandle<'a> for bool {
    type Handle = JsBoolean;
    fn into_handle(&self, cx: &mut impl Context<'a>) -> NeonResult<Handle<'a, Self::Handle>> {
        Ok(cx.boolean(*self))
    }
}

impl FromHandle for Vec<u8> {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let receipt = String::from_handle(handle, cx)?;
        receipt.from_hex().js_map_err(cx, |_| "Invalid hex")
    }
}

impl FromHandle for u64 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let number = f64::from_handle(handle, cx)?;
        if number.is_nan()
            || number.is_infinite()
            || number < 0.0
            || number.fract() != 0.0
            || number > 9007199254740991.0
        {
            throw(
                cx,
                format!("Expecting integer block number. Got {}", number),
            )
        } else {
            Ok(number as u64)
        }
    }
}

impl FromHandle for Duration {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let ms = f64::from_handle(handle, cx)?;

        if ms < 0.0 || ms.is_infinite() || ms.is_nan() {
            throw(cx, format!("Expecting finite duration >= 0 ms. Got {}", ms))
        } else {
            Ok(Duration::from_secs_f64(ms / 1000.0))
        }
    }
}

impl FromHandle for Bytes32 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        decode(s.as_str()).js_map_err(cx, |_| format!("Failed to parse Bytes32 from \"{}\"", s))
    }
}

impl FromHandle for U256 {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        decode(&s).js_map_err(cx, |_| format!("Failed to parse U256 from, {}", s))
    }
}

impl FromHandle for SecretKey {
    fn from_handle<'a, V: Value>(handle: Handle<V>, cx: &mut impl Context<'a>) -> NeonResult<Self>
    where
        Self: Sized,
    {
        let s = String::from_handle(handle, cx)?;
        s.parse().js_map_err(cx, |_| "Failed to parse secret key")
    }
}
