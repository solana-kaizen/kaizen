use std::sync::Arc;
use crate::context::HandlerFn;
use crate::builder::InstructionBuilder;

pub trait Client {
    fn handler_id(handler_fn: HandlerFn) -> usize;
    fn execution_context_for(handler: HandlerFn) -> Arc<InstructionBuilder>;
}

