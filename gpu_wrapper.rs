use cust::prelude::*;
use cust::memory::*;
use cust::stream::StreamFlags;
use std::sync::Arc;

static PTX: &str = include_str!("../gpu_kernel.ptx");

static mut CONTEXT: Option<Arc<Context>> = None;
static mut MODULE: Option<Module> = None;

pub fn init_gpu(_words: &Vec<String>, _known: &[&str], _target: &str) {
    let ctx = cust::quick_init().expect("CUDA init failed");
    let module = Module::from_ptx(PTX, &[]).expect("PTX load failed");

    unsafe {
        CONTEXT = Some(Arc::new(ctx));
        MODULE = Some(module);
    }
}

// NEW VERSION
pub fn launch_batch(worker_id: u32, total_workers: u32, match_mode: i32, match_prefix_len: i32) -> (u64, u64)
 {
    unsafe {
        let context = CONTEXT.as_ref().expect("Context missing");
        let module = MODULE.as_ref().expect("Module missing");

        let function = module.get_function("search_seeds").expect("Kernel not found");

        let seeds_tested = DeviceBuffer::<u64>::zeroed(1).unwrap();
        let seeds_found = DeviceBuffer::<u64>::zeroed(1).unwrap();
        let precomputed = DeviceBuffer::<u16>::zeroed(1).unwrap();
        let target_address = DeviceBuffer::<u8>::zeroed(20).unwrap();

        let threads_per_block = 512u64;
        let total_candidates: u64 = 10_000_000;
        let blocks = (total_candidates + threads_per_block - 1) / threads_per_block;

        let stream = Stream::new(StreamFlags::DEFAULT, None).unwrap();

        // 🛠 Launch with worker_id passed in
        launch!(
    function<<<blocks as u32, threads_per_block as u32, 0, stream>>>(
        seeds_tested.as_device_ptr(),
        seeds_found.as_device_ptr(),
        precomputed.as_device_ptr(),
        total_candidates as u32,
        target_address.as_device_ptr(),
        worker_id,
        total_workers,
        match_mode,
        match_prefix_len
    )
)

        ).unwrap();

        stream.synchronize().unwrap();

        let mut tested = [0u64];
        let mut found = [0u64];
        seeds_tested.copy_to(&mut tested).unwrap();
        seeds_found.copy_to(&mut found).unwrap();

        (tested[0], found[0])
    }
}
