pub mod decomp_error;
pub mod disassembler;
pub mod ir_to_source;
pub mod levelscript;

use std::collections::HashMap;

use crate::{ConstantDb, DatabaseV2, ProjectContext};

/// Borrowed resources used to render disassembled scripts.
#[derive(Clone, Copy)]
pub struct DecompileContext<'a> {
    db: &'a DatabaseV2,
    constants: Option<&'a ConstantDb>,
    project: Option<&'a ProjectContext>,
    /// Batch global file ID -> zero-based slot -> emitted reference label.
    /// A selected file with no valid headers has an empty map, not stale source names.
    pub(crate) global_script_names: Option<&'a HashMap<u16, HashMap<u32, String>>>,
}

impl<'a> DecompileContext<'a> {
    /// Use the database and project-wide constants from a loaded project.
    pub fn for_project(project: &'a ProjectContext) -> Self {
        Self {
            db: project.db(),
            constants: Some(project.project_constants()),
            project: Some(project),
            global_script_names: None,
        }
    }

    /// Use explicitly supplied resources without project-backed resolution.
    pub fn standalone(db: &'a DatabaseV2, constants: Option<&'a ConstantDb>) -> Self {
        Self {
            db,
            constants,
            project: None,
            global_script_names: None,
        }
    }

    pub(crate) fn db(self) -> &'a DatabaseV2 {
        self.db
    }

    pub(crate) fn constants(self) -> Option<&'a ConstantDb> {
        self.constants
    }

    pub(crate) fn project(self) -> Option<&'a ProjectContext> {
        self.project
    }
}

pub use decomp_error::{DecompileError, DecompileResult};
pub use disassembler::{Disassembler, ScriptOutput, ScriptType, disassemble_bytes};
pub use ir_to_source::ir_to_source;
pub use levelscript::{
    LevelScript, LevelScriptHeaderEntry, LevelScriptValidationError, LevelScriptVarConditionEntry,
};
