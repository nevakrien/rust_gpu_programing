use wgpu::util::DeviceExt;
use futures::executor::block_on;


use tokio::sync::oneshot;
#[tokio::main]
async fn main() {
//fn main(){    
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    // let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    //     backends: wgpu::Backends::all(),
    //     ..Default::default()
    // });

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });


    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })).unwrap();

    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    )).unwrap();

    // Load the shader
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compute Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("test_shader.wgsl").into()),
    });

    // Setup the compute pipeline
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Compute Bind Group Layout"),
        entries: &[
            // Input buffer
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Output buffer
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Scalar uniform
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Compute Pipeline Layout"),
        bind_group_layouts: &[&bind_group_layout/* Bind group layouts here */],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Compute Pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    // Create buffers and bind groups as needed, dispatch the compute work, etc.
    const buffer_size :usize=4;
    let input_data = vec![1.0_f32; buffer_size]; // Adjust the size as needed
    let scalar_value = 2.0_f32;

    // Create the input buffer
    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Input Buffer"),
        contents: bytemuck::cast_slice(&input_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });

    // Create the output buffer
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (input_data.len() * std::mem::size_of::<f32>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC, //| wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    // Create the buffer for the scalar value
    let scalar_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Scalar Buffer"),
        contents: bytemuck::cast_slice(&[scalar_value]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            // Input buffer binding
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            // Output buffer binding
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buffer.as_entire_binding(),
            },
            // Scalar uniform binding
            wgpu::BindGroupEntry {
                binding: 2,
                resource: scalar_buffer.as_entire_binding(),
            },
        ],
        label: Some("Compute Bind Group"),
    });

    //run
    let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Command Encoder"),
        });
    {
        
        
        let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None
        });
        compute_pass.set_pipeline(&compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        // Define the number of workgroups
        //compute_pass.dispatch(workgroups_x, workgroups_y, workgroups_z);
        compute_pass.dispatch_workgroups(16, 1, 1);
    }
    queue.submit(Some(command_encoder.finish()));


    // //print the output...

    // let (sender, receiver) = oneshot::channel();//futures_channel::oneshot::channel();
    // output_buffer
    //     .slice(..)
    //     .map_async(wgpu::MapMode::Read, |result| {
    //         let _ = sender.send(result);
    //     });
    // device.poll(wgpu::Maintain::Wait); // TODO: poll in the background instead of blocking
    // receiver
    //     .await
    //     .expect("communication failed")
    //     .expect("buffer reading failed");
    // let slice: &[u8] = &output_buffer.slice(..).get_mapped_range();
    // println!("Output: {:?}", slice);


    // Step 1: Create a staging buffer
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: (buffer_size*std::mem::size_of::<f32>()) as u64, // Match the size of the data you want to copy
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST, // For reading back and as a copy destination
        mapped_at_creation: false,
    });

    // Step 2: Copy data to the staging buffer
    {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });
        encoder.copy_buffer_to_buffer(
            &output_buffer, // Source buffer
            0, // Source offset
            &staging_buffer, // Destination buffer
            0, // Destination offset
            (buffer_size*std::mem::size_of::<f32>()) as u64, // Number of bytes to copy
        );
        queue.submit(Some(encoder.finish()));
    }

    let answer: Vec<f32>;
    // Use the oneshot channel and polling as in your snippet to wait for the mapping operation
    {
        let (sender, receiver) = oneshot::channel();
        staging_buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });
        device.poll(wgpu::Maintain::Wait); // In practice, consider polling in a non-blocking manner
        receiver.await.expect("communication failed").expect("buffer reading failed");

        // Now you can read from the staging_buffer
        let data_range = staging_buffer.slice(..).get_mapped_range();
        // Use bytemuck to safely cast the byte slice to an f32 slice
        let data: &[f32] = bytemuck::cast_slice(&data_range);
        //println!("Output: {:?}", data);
        answer= data.to_vec();

    }
    
    // Remember to unmap the buffer when done
    staging_buffer.unmap();
    println!("Output: {:?}", answer);

}
