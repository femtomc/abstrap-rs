/*!

   The builder design in this module
   supports code generation to the `abstraps` IR.

   The interfaces provided below allow customized code generation
   for user-defined intrinsics and lowering.

*/

use crate::ir::core::{Attribute, BasicBlock, Intrinsic, IntrinsicTrait, Operation, Region, Var};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use anyhow::Result;
use serde::{Serialize, Serializer};
use std::collections::HashMap;

#[derive(Debug)]
pub enum BuilderError {
    BuilderCreationFailure,
    Caseless,
}

#[derive(Debug)]
pub struct OperationBuilder {
    latest: Vec<Var>,
    cursor: (usize, usize),
    intrinsic: Box<dyn Intrinsic>,
    operands: Vec<Var>,
    attrs: HashMap<String, Box<dyn Attribute>>,
    regions: Vec<Region>,
    successors: Vec<BasicBlock>,
}

impl OperationBuilder {
    pub fn default(intr: Box<dyn Intrinsic>) -> OperationBuilder {
        OperationBuilder {
            latest: Vec::new(),
            cursor: (0, 0),
            intrinsic: intr,
            operands: Vec::new(),
            attrs: HashMap::new(),
            regions: Vec::new(),
            successors: Vec::new(),
        }
    }

    pub fn get_latest(&self) -> Vec<Var> {
        self.latest.to_vec()
    }

    pub fn get_intrinsic(&self) -> &Box<dyn Intrinsic> {
        &self.intrinsic
    }

    pub fn push_operand(&mut self, arg: Var) {
        self.operands.push(arg);
    }

    pub fn set_operands(&mut self, args: Vec<Var>) {
        self.operands = args;
    }

    pub fn get_operands(&self, arg: Var) -> Vec<Var> {
        self.operands.to_vec()
    }

    pub fn set_cursor(&mut self, reg: usize, blk: usize) {
        self.cursor = (reg, blk);
    }

    pub fn get_cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn push_arg(&mut self) -> Result<Var> {
        let blk = self.cursor.1 - 1;
        let r = self.get_region();
        match r.push_arg(blk) {
            Ok(v) => {
                if blk == 0 {
                    self.push_operand(v);
                }
                Ok(v)
            }
            Err(e) => Err(e),
        }
    }

    pub fn insert_attr(&mut self, k: &str, attr: Box<dyn Attribute>) {
        self.attrs.insert(k.to_string(), attr);
    }

    pub fn get_attrs(&self) -> &HashMap<String, Box<dyn Attribute>> {
        &self.attrs
    }

    pub fn get_attr(&self, key: &str) -> Option<&Box<dyn Attribute>> {
        self.attrs.get(key)
    }

    pub fn push_region(&mut self, r: Region) {
        self.regions.push(r);
        self.cursor = (self.cursor.0 + 1, self.cursor.1)
    }

    pub fn get_region(&mut self) -> &mut Region {
        let reg = self.cursor.0 - 1;
        &mut self.regions[reg]
    }

    pub fn get_regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn push_block(&mut self, b: BasicBlock) -> Result<()> {
        let r = self.get_region();
        r.push_block(b)?;
        self.cursor = (self.cursor.0, self.cursor.1 + 1);
        Ok(())
    }

    pub fn get_block(&mut self) -> &mut BasicBlock {
        let cursor = self.cursor;
        let blk = cursor.1 - 1;
        let b = self.get_region().get_block(blk);
        return b;
    }

    pub fn push_op(mut self, v: Operation) -> OperationBuilder {
        let ret = {
            let blk = self.get_cursor().1 - 1;
            let r = self.get_region();
            r.push_op(blk, v)
        };
        self.latest = vec![ret];
        self
    }
}

impl OperationBuilder {
    pub fn finish(self) -> Result<Operation> {
        Ok(Operation::new(
            self.intrinsic,
            self.operands,
            self.attrs,
            self.regions,
            self.successors,
        ))
    }
}
