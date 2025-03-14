mod store;
mod synthesis;

use lmt_parser::SimpleSpan;
pub use store::{ModuleId, ModuleMap};
pub use synthesis::ModuleSynthesis;

pub type SpanWithModuleId = SimpleSpan<usize, ModuleId>;
