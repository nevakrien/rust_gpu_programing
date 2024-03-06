@group(0) @binding(0) var<storage, read> inputBuffer: array<f32>;
//@group(0) @binding(1) var<storage, write> outputBuffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> outputBuffer: array<f32>;
@group(0) @binding(2) var<uniform> scalar: f32;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index: u32 = global_id.x;
    outputBuffer[index] = inputBuffer[index] * scalar;
}
