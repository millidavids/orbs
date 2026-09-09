//! Does a tensor actually run on this machine's GPU, and can burn share a
//! device with the renderer?
//!
//! **The spike DESIGN.md §19 asks for before a model is built.** Two questions,
//! and the second is the one that decides whether the augury costs a second
//! Vulkan stack:
//!
//! 1. Does `burn` compute anything at all on this GPU, through Vulkan?
//! 2. Can it be handed a device somebody else made, rather than making its own?
//!
//! Not a test, because it needs a GPU and `cargo test --workspace` must run on
//! a machine without one. Run it by hand:
//!
//! ```text
//! cargo run -p orbs-augury --example device --features train
//! ```

use burn::backend::wgpu::{Wgpu, WgpuDevice};
use burn::tensor::Tensor;

fn main() {
    println!("\nO.R.B.S. — can the augury reach the GPU?\n");

    // **A default device makes its own adapter**, which is what a standalone
    // trainer wants and what the game must *not* do — the game already has one.
    let device = WgpuDevice::default();
    println!("  device        {device:?}");

    // The smallest thing that proves a kernel was compiled, dispatched and read
    // back. A matmul rather than an add: it is what an encoder is made of, and
    // an add can be constant-folded away.
    let left: Tensor<Wgpu, 2> = Tensor::from_floats([[1.0, 2.0], [3.0, 4.0]], &device);
    let right: Tensor<Wgpu, 2> = Tensor::from_floats([[5.0, 6.0], [7.0, 8.0]], &device);
    let product = left.matmul(right);

    let data = product.to_data();
    println!("  matmul        {data:?}");

    let expected = [19.0f32, 22.0, 43.0, 50.0];
    let got = data.as_slice::<f32>().expect("f32 out");
    assert_eq!(got, expected, "the GPU did the arithmetic wrong");
    println!("  ...on a device of its own making. Now somebody else's.\n");

    adopted();
}

/// Hand burn a device made outside it, the way the game will.
///
/// **The question the spike exists for.** `orbs` already has a wgpu device —
/// Bevy made it, and every frame of the tube runs on it. If burn cannot be
/// given that one it opens a second, which means a second Vulkan instance and a
/// second shader compiler in a game whose whole picture is a text grid.
///
/// Bevy exposes all four pieces (`RenderInstance`, `RenderAdapter`,
/// `RenderDevice::wgpu_device`, `RenderQueue`); this makes them the same way
/// Bevy does, so what is proved here is the *mechanism* rather than the plumbing.
fn adopted() {
    use burn::backend::wgpu::{RuntimeOptions, WgpuSetup, init_device};

    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("no adapter");
    let info = adapter.get_info();
    println!("  adapter       {} ({:?})", info.name, info.backend);

    // **Not `DeviceDescriptor::default()`.** That asks for no features and the
    // downlevel limits, and a device built that way is *accepted* by
    // `init_device` and then computes zeros — see the §19 entry. Ask the adapter
    // for everything it has, which is the closest stand-in for a renderer that
    // configured its own device deliberately.
    // **Not `DeviceDescriptor::default()`.** That asks for the *downlevel*
    // limits, and a device built that way is accepted by `init_device` and then
    // computes zeros — see the §19 entry. The adapter's own limits are what a
    // renderer that configured a device deliberately would have.
    //
    // Features are left alone: asking for `adapter.features()` wholesale fails,
    // because on this driver that set includes six `EXPERIMENTAL_*` flags that
    // need a separate opt-in nobody wants turned on by accident.
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("orbs-augury spike"),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .expect("no device");

    let shared = init_device(
        WgpuSetup {
            instance,
            adapter,
            device,
            queue,
            backend: info.backend,
        },
        RuntimeOptions::default(),
    );
    println!("  shared device {shared:?}");

    let left: Tensor<Wgpu, 2> = Tensor::from_floats([[1.0, 2.0], [3.0, 4.0]], &shared);
    let right: Tensor<Wgpu, 2> = Tensor::from_floats([[5.0, 6.0], [7.0, 8.0]], &shared);
    let data = left.matmul(right).to_data();
    let got = data.as_slice::<f32>().expect("f32 out");
    assert_eq!(
        got,
        [19.0f32, 22.0, 43.0, 50.0],
        "a shared device did the arithmetic wrong"
    );

    println!("\n  a device made elsewhere computes. One Vulkan stack, not two.");
    hazard();
}

/// The way this goes wrong, kept because it goes wrong *quietly*.
///
/// **A device built with `DeviceDescriptor::default()` is accepted by
/// `init_device` and then computes zeros.** No error, no warning, no panic —
/// `request_device` succeeds, the setup is taken, the tensor round-trips, and
/// every number in it is 0.0. The difference is `required_limits`: the default
/// is the *downlevel* set, and cubecl's kernels need what the adapter actually
/// has.
///
/// This matters for the real integration rather than for the spike. Bevy
/// configures its own device, so whether the augury works at all will depend on
/// limits chosen elsewhere in the frontend for reasons that have nothing to do
/// with it — and the failure will look like a model that reads every sentence
/// as the same command, not like a device problem.
///
/// **So adopting a device has to be checked, not assumed.** A known matmul with
/// a known answer, once, at startup, and fall back to the CPU backend when it
/// fails. Demonstrated here so the check has something to be written against.
fn hazard() {
    use burn::backend::wgpu::{RuntimeOptions, WgpuSetup, init_device};

    let instance = wgpu::Instance::default();
    let Ok(adapter) =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
    else {
        return;
    };
    let backend = adapter.get_info().backend;
    let Ok((device, queue)) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
    else {
        return;
    };

    let downlevel = init_device(
        WgpuSetup {
            instance,
            adapter,
            device,
            queue,
            backend,
        },
        RuntimeOptions::default(),
    );

    let left: Tensor<Wgpu, 2> = Tensor::from_floats([[1.0, 2.0], [3.0, 4.0]], &downlevel);
    let right: Tensor<Wgpu, 2> = Tensor::from_floats([[5.0, 6.0], [7.0, 8.0]], &downlevel);
    let data = left.matmul(right).to_data();
    let got = data.as_slice::<f32>().unwrap_or(&[]);

    println!("\n  and the trap, for the record:");
    println!("    default limits -> {got:?}   (should be [19, 22, 43, 50])");
    if got.first().is_some_and(|first| *first == 19.0) {
        println!("    ...which passed here. The check is still owed — it failed once.");
    } else {
        println!("    silent zeros. No error, no warning. Verify a device before trusting it.");
    }
    println!();
}
