#[cfg(not(feature = "dev"))]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    omnihilux::entry();
}
