use std::sync::{Mutex, OnceLock, atomic::{AtomicUsize, Ordering}, mpsc::{self, Sender, Receiver}};
use std::thread::{self, JoinHandle};

pub mod ast;
pub mod cli;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod package;
pub mod registry;
pub mod typecheck;
use std::collections::HashMap as Map;

static THREADS: OnceLock<Mutex<Map<usize, JoinHandle<Value>>>> = OnceLock::new();
pub(crate) static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(1);
pub(crate) static NEXT_CHANNEL_ID: AtomicUsize = AtomicUsize::new(1);
static CHANNELS: OnceLock<Mutex<Map<usize, (Sender<Value>, Receiver<Value>)>>> = OnceLock::new();
pub(crate) fn channel_table() -> &'static Mutex<Map<usize, (Sender<Value>, Receiver<Value>)>> { CHANNELS.get_or_init(|| Mutex::new(Map::new())) }
pub(crate) fn thread_table() -> &'static Mutex<Map<usize, JoinHandle<Value>>> { THREADS.get_or_init(|| Mutex::new(Map::new())) }


pub use ast::{Expr, Function, Program, Stmt, Type, Value};
pub use lexer::{lex, Token};
pub use runtime::{build_artifact, execute, BytecodeVm};


