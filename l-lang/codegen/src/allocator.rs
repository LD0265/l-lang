use std::collections::HashMap;

use ir::instruction::IrType;
use semantic::symbol::SymbolId;

pub struct Allocator {
    stack_variables: HashMap<SymbolId, usize>,
    ra_offset: Option<usize>,
    stack_size: usize,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            stack_variables: HashMap::new(),
            ra_offset: None,
            stack_size: 0,
        }
    }

    pub fn get_variable_offset(&self, id: &SymbolId) -> usize {
        *self.stack_variables.get(id).unwrap_or_else(|| {
            panic!(
                "no stack slot for SymbolId({:?}) — was it ever allocated?",
                id.0
            )
        })
    }

    pub fn get_ra_offset(&self) -> Option<usize> {
        self.ra_offset
    }

    pub fn get_stack_size(&self) -> usize {
        self.stack_size
    }

    pub fn insert_variable(&mut self, id: &SymbolId, ir_type: &IrType) {
        let size = ir_type.size_bytes();
        // align current offset to this type's size
        if size > 1 {
            self.stack_size = (self.stack_size + size - 1) & !(size - 1);
        }
        self.stack_variables.insert(id.clone(), self.stack_size);
        self.stack_size += size;
    }

    pub fn insert_ra(&mut self) {
        // align to 4 bytes before storing $ra
        self.stack_size = (self.stack_size + 3) & !3;
        self.ra_offset = Some(self.stack_size);
        self.stack_size += 4;
    }

    // needed for mips apparently
    pub fn align8(&mut self) {
        self.stack_size = (self.stack_size + 7) & !7
    }
}
