#[macro_use]
extern crate neon;
extern crate polyline;

use neon::vm::{Call, JsResult, Module};
use neon::js::JsString;
use neon::js::JsNumber;
use neon::js::JsArray;
use neon::mem::Handle;

fn decode(call: Call) -> JsResult<JsArray> {
    let scope = call.scope;
    let input: Handle<JsString> = try!(try!(call.arguments.require(scope, 0)).check::<JsString>());
    let precision: Handle<JsNumber> = try!(try!(call.arguments.require(scope, 1)).check::<JsNumber>());
    let decoded = polyline::decode_polyline(input.value(), precision.value() as u32);

    let array: Handle<JsArray> = JsArray::new(scope, decoded.len() as u32);

    for (idx, coord) in decoded.into_iter().enumerate() {
        let c: Handle<JsArray> = JsArray::new(scope, 2);
        try!(c.set(0, JsNumber::new(scope, coord[0])));
        try!(c.set(1, JsNumber::new(scope, coord[0])));
        try!(array.set(idx as i32, c));
    }

    Ok(array)
}

register_module!(m, {
    m.export("decode", decode)
});
