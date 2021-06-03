//! This example describes how to configure parameters of a camera.
use cameleon::u3v;

fn main() {
    let mut cameras = u3v::enumerate_cameras().unwrap();
    if cameras.is_empty() {
        return;
    }
    let mut camera = cameras.pop().unwrap();
    // Loads `GenApi` context.
    camera.load_context().unwrap();

    let mut params_ctxt = camera.params_ctxt().unwrap();
    // Get `Gain` node of `GenApi`.
    // `GenApi SFNC` defines that `Gain` node should have `IFloat` interface,
    // so this conversion would be success if the camera follows that.
    // Some vendors may define `Gain` node as `IInteger`, in that case, use
    // `as_integer(&params_ctxt)` instead of `as_float(&params_ctxt).
    let gain_node = params_ctxt
        .node("Gain")
        .unwrap()
        .as_float(&params_ctxt)
        .unwrap();

    // Get the current value of `Gain`.
    if gain_node.is_readable(&mut params_ctxt).unwrap() {
        let value = gain_node.value(&mut params_ctxt).unwrap();
        println!("{}", value);
    }

    // Set `0.1` to `Gain`.
    if gain_node.is_writable(&mut params_ctxt).unwrap() {
        gain_node.set_value(&mut params_ctxt, 0.1).unwrap();
    }
}
