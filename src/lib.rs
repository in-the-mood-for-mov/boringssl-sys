#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::pthread_rwlock_t;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
