use napi_derive::napi;

#[global_allocator]
static ALLOC: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;

#[napi]
pub fn hello() -> String {
    "Hello, world!".to_string()
}
