use std::collections::HashMap;

use ir::instruction::IrType;
use semantic::symbol::SymbolId;

pub struct Allocator {
    stack_variables: HashMap<SymbolId, usize>,
    spill_slots: HashMap<usize, usize>,
    array_slots: HashMap<usize, usize>, // slot -> base offset
    ra_offset: Option<usize>,
    stack_size: usize,
}

impl Allocator {
    pub fn new() -> Self {
        Self {
            stack_variables: HashMap::new(),
            spill_slots: HashMap::new(),
            array_slots: HashMap::new(),
            ra_offset: None,
            stack_size: 0,
        }
    }

    pub fn insert_array(&mut self, slot: usize, elem_size: usize, count: usize) {
        // align to elem size
        if elem_size > 1 {
            self.stack_size = (self.stack_size + elem_size - 1) & !(elem_size - 1);
        }
        self.array_slots.insert(slot, self.stack_size);
        self.stack_size += elem_size * count;
    }

    pub fn get_array_base_offset(&self, slot: usize) -> usize {
        *self.array_slots.get(&slot).unwrap_or_else(|| {
            panic!(
                "no stack slot for array slot {} — was it ever allocated?",
                slot
            )
        })
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

    pub fn get_or_insert_spill(&mut self, slot: usize) -> usize {
        if let Some(&offset) = self.spill_slots.get(&slot) {
            return offset;
        }
        // align to 4
        self.stack_size = (self.stack_size + 3) & !3;
        let offset = self.stack_size;
        self.spill_slots.insert(slot, offset);
        self.stack_size += 4;
        offset
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

    pub fn insert_struct(&mut self, id: &SymbolId, size_bytes: usize) {
        self.stack_size = (self.stack_size + 3) & !3;
        self.stack_variables.insert(id.clone(), self.stack_size);
        self.stack_size += size_bytes;
    }

    // needed for mips apparently
    pub fn align8(&mut self) {
        self.stack_size = (self.stack_size + 7) & !7
    }
}
