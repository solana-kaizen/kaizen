use crate::builder::InstructionBuilder;
use crate::context::HandlerFn;
use std::sync::Arc;

pub trait Client {
    fn handler_id(handler_fn: HandlerFn) -> usize;
    fn execution_context_for(handler: HandlerFn) -> Arc<InstructionBuilder>;
}
