use std::ops::{Index, IndexMut};

use isa::{CompiledProgram, CompiledSection, MemoryItem, RawAddress, MEMORY_SIZE};

type TMemory = Vec<MemoryItem>;
pub struct Memory(TMemory);

impl Index<RawAddress> for Memory {
    type Output = MemoryItem;

    fn index(&self, index: RawAddress) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<RawAddress> for Memory {
    fn index_mut(&mut self, index: RawAddress) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl Index<usize> for Memory {
    type Output = MemoryItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Memory {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Memory {
    /// Creates empty memory
    pub fn new() -> Self {
        Self(TMemory::with_capacity(MEMORY_SIZE))
    }

    /// Creates new memory and burns program into it
    pub fn burn(CompiledProgram { sections }: CompiledProgram) -> Memory {
        let mut memory: Memory = Self::new();

        for section in sections {
            memory.burn_section(section);
        }

        memory
    }

    fn burn_section(
        &mut self,
        CompiledSection {
            start_address,
            items,
        }: CompiledSection,
    ) {
        // splice *inserts* elements rather than replace
        for (offset, item) in items.iter().enumerate() {
            self[start_address + offset as u16] = *item;
        }
    }
}
