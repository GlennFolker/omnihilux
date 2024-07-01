#[cfg(all(not(feature = "dev"), not(target_arch = "wasm32")))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    omnihilux::entry();
}
