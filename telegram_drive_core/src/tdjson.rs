use std::ffi::{c_char, c_double, c_int, c_void};

#[link(name = "tdjson")]
extern "C" {
    pub fn td_create_client_id() -> c_int;
    pub fn td_send(client_id: c_int, request: *const c_char) -> c_void;
    pub fn td_receive(timeout: c_double) -> *const c_char;
    pub fn td_execute(request: *const c_char) -> *const c_char;
    pub fn td_set_log_message_callback(
        max_verbosity_level: c_int,
        callback: extern "C" fn(c_int, *const c_char) -> c_void
    ) -> c_void;
    pub fn td_json_client_create() -> *const c_void;
    pub fn td_json_client_send(client: *const c_void, request: *const c_char) -> c_void;
    pub fn td_json_client_receive(client: *const c_void, timeout: c_double) -> *const c_char;
    pub fn td_json_client_execute(client: *const c_void, request: *const c_char) -> *const c_char;
    pub fn td_json_client_destroy(client: *const c_void) -> c_void;
}

fn td_log_message_callback_ptr(verbosity_level: c_int, message: *const c_char) -> () {
    println!("{:?} - {:?}", verbosity_level, message);
    //return c_void
}