use std::{collections::HashMap, sync::Arc};

use crate::{
    asm_generation::{
        asm_builder::{AsmBuilder, AsmBuilderResult},
        from_ir::StateAccessType,
        ProgramKind,
    },
    asm_lang::Label,
    error::*,
    metadata::MetadataManager,
};
use etk_ops::london::*;
use sway_error::error::CompileError;
use sway_ir::{Context, *};
use sway_types::Span;

use etk_asm::{asm::Assembler, ops::*};

/// A smart contract is created by sending a transaction with an empty "to" field.
/// When this is done, the Ethereum virtual machine (EVM) runs the bytecode which is
/// set in the init byte array which is a field that can contain EVM bytecode
///
/// The EVM bytecode that is then stored on the blockchain is the value that is
/// returned by running the content of init on the EVM.
///
/// The bytecode can refer to itself through the opcode CODECOPY opcode, which reads
/// three values on the stack where two of those values are pointers to the bytecode,
/// one marking the beginning and one marking the end of what should be copied to memory.
///
/// The RETURN opcode is then used, along with the correct values placed on the stack,
/// to return bytecode from the initial run of the EVM code.
/// RETURN reads and removes two pointers from the stack.
/// These pointers define the part of the memory that is a return value.
/// The return value of the initial contract creating run of the bytecode defines
/// the bytecode that is stored on the blockchain and associated with the address
/// on which you have created the smart contract.
///
/// The code that is compiled but not stored on the blockchain is thus the code needed
/// to store the correct code on the blockchain but also any logic that is contained in
/// a (potential) constructor of the contract.

pub struct EvmAsmBuilder<'ir> {
    #[allow(dead_code)]
    program_kind: ProgramKind,

    sections: Vec<EvmAsmSection>,

    // Label maps are from IR functions or blocks to label name.  Functions have a start and end
    // label.
    pub(super) func_label_map: HashMap<Function, (Label, Label)>,
    #[allow(dead_code)]
    pub(super) block_label_map: HashMap<Block, Label>,

    // IR context we're compiling.
    context: &'ir Context,

    // Metadata manager for converting metadata to Spans, etc.
    md_mgr: MetadataManager,

    // Monotonically increasing unique identifier for label generation.
    label_idx: usize,

    // In progress EVM asm section.
    pub(super) cur_section: Option<EvmAsmSection>,
}

#[derive(Default, Debug)]
pub struct EvmAsmSection {
    ops: Vec<etk_asm::ops::AbstractOp>,
    abi: Vec<ethabi::operation::Operation>,
}

impl EvmAsmSection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(&self) -> usize {
        let mut asm = Assembler::new();
        if asm.push_all(self.ops.clone()).is_err() {
            panic!("Could not size EVM assembly section");
        }
        asm.take().len()
    }
}

pub struct EvmAsmBuilderResult {
    pub ops: Vec<etk_asm::ops::AbstractOp>,
    pub ops_runtime: Vec<etk_asm::ops::AbstractOp>,
    pub abi: EvmAbiResult,
}

pub type EvmAbiResult = Vec<ethabi::operation::Operation>;

impl<'ir> AsmBuilder for EvmAsmBuilder<'ir> {
    fn func_to_labels(&mut self, func: &Function) -> (Label, Label) {
        self.func_to_labels(func)
    }

    fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        self.compile_function(function)
    }

    fn finalize(&self) -> AsmBuilderResult {
        self.finalize()
    }
}

#[allow(unused_variables)]
#[allow(dead_code)]
impl<'ir> EvmAsmBuilder<'ir> {
    pub fn new(program_kind: ProgramKind, context: &'ir Context) -> Self {
        Self {
            program_kind,
            sections: Vec::new(),
            func_label_map: HashMap::new(),
            block_label_map: HashMap::new(),
            context,
            md_mgr: MetadataManager::default(),
            label_idx: 0,
            cur_section: None,
        }
    }

    pub fn finalize(&self) -> AsmBuilderResult {
        let mut global_ops = Vec::new();
        let mut global_abi = Vec::new();

        let mut size = 0;
        let mut it = self.sections.iter().peekable();
        while let Some(section) = it.next() {
            size += section.size();
            global_ops.append(&mut section.ops.clone());
            global_abi.append(&mut section.abi.clone());

            if it.peek().is_some() {
                size += AbstractOp::Op(Op::Invalid(etk_ops::london::Invalid))
                    .size()
                    .unwrap();
                global_ops.push(AbstractOp::Op(Op::Invalid(etk_ops::london::Invalid)));
            }
        }

        // First generate a dummy ctor section to calculate its size.
        let dummy = self.generate_constructor(false, size, 0);

        // Generate the actual ctor section with the correct size..
        let mut ctor = self.generate_constructor(false, size, dummy.size());
        ctor.ops.append(&mut global_ops);
        global_abi.append(&mut ctor.abi);

        AsmBuilderResult::Evm(EvmAsmBuilderResult {
            ops: ctor.ops.clone(),
            ops_runtime: ctor.ops,
            abi: global_abi,
        })
    }

    fn generate_constructor(
        &self,
        is_payable: bool,
        data_size: usize,
        data_offset: usize,
    ) -> EvmAsmSection {
        // For more details and explanations see:
        // https://medium.com/@hayeah/diving-into-the-ethereum-vm-part-5-the-smart-contract-creation-process-cb7b6133b855.

        let mut s = EvmAsmSection::new();
        self.setup_free_memory_pointer(&mut s);

        if is_payable {
            // Get the the amount of ETH transferred to the contract by the parent contract,
            // or by a transaction and check for a non-payable contract. Revert if caller
            // sent ether.
            //
            //   callvalue
            //   dup1
            //   iszero
            //   push1 0x0f
            //   jumpi
            //   push1 0x00
            //   dup1
            //   revert
            //   jumpdest
            //   pop

            s.ops.push(AbstractOp::new(Op::CallValue(CallValue)));
            s.ops.push(AbstractOp::new(Op::Dup1(Dup1)));
            s.ops.push(AbstractOp::new(Op::IsZero(IsZero)));
            let tag_label = "tag_1";
            s.ops.push(AbstractOp::new(Op::Push1(Push1(Imm::with_label(
                tag_label,
            )))));
            s.ops.push(AbstractOp::new(Op::JumpI(JumpI)));
            s.ops
                .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                    Expression::Terminal(0x00.into()),
                )))));
            s.ops.push(AbstractOp::new(Op::Dup1(Dup1)));
            s.ops.push(AbstractOp::new(Op::Revert(Revert)));

            s.ops.push(AbstractOp::Label("tag_1".into()));
            s.ops.push(AbstractOp::new(Op::JumpDest(JumpDest)));
            s.ops.push(AbstractOp::Op(Op::Pop(Pop)));
        }

        self.copy_contract_code_to_memory(&mut s, data_size, data_offset);

        s.abi.push(ethabi::operation::Operation::Constructor(
            ethabi::Constructor { inputs: vec![] },
        ));

        s
    }

    fn copy_contract_code_to_memory(
        &self,
        s: &mut EvmAsmSection,
        data_size: usize,
        data_offset: usize,
    ) {
        // Copy contract code into memory, and return.
        //   push1 dataSize
        //   dup1
        //   push1 dataOffset
        //   push1 0x00
        //   codecopy
        //   push1 0x00
        //   return
        s.ops.push(AbstractOp::Push(Imm::from(Terminal::Number(
            data_size.into(),
        ))));
        s.ops.push(AbstractOp::new(Op::Dup1(Dup1)));
        s.ops.push(AbstractOp::Push(Imm::from(Terminal::Number(
            data_offset.into(),
        ))));
        s.ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x00.into()),
            )))));
        s.ops.push(AbstractOp::Op(Op::CodeCopy(CodeCopy)));

        s.ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x00.into()),
            )))));
        s.ops.push(AbstractOp::Op(Op::Return(Return)));
    }

    fn setup_free_memory_pointer(&self, s: &mut EvmAsmSection) {
        // Setup the initial free memory pointer.
        //
        // The "free memory pointer" is stored at position 0x40 in memory.
        // The first 64 bytes of memory can be used as "scratch space" for short-term allocation.
        // The 32 bytes after the free memory pointer (i.e., starting at 0x60) are meant to be
        // zero permanently and is used as the initial value for empty dynamic memory arrays.
        // This means that the allocatable memory starts at 0x80, which is the initial value
        // of the free memory pointer.
        //
        //   push1 0x80
        //   push1 0x40
        //   mstore

        s.ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x80.into()),
            )))));
        s.ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x40.into()),
            )))));
        s.ops.push(AbstractOp::new(Op::MStore(MStore)));
    }

    fn empty_span() -> Span {
        let msg = "unknown source location";
        Span::new(Arc::from(msg), 0, msg.len(), None).unwrap()
    }

    fn get_label(&mut self) -> Label {
        let next_val = self.label_idx;
        self.label_idx += 1;
        Label(self.label_idx)
    }

    pub(super) fn compile_instruction(
        &mut self,
        instr_val: &Value,
        func_is_entry: bool,
    ) -> CompileResult<()> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        if let Some(instruction) = instr_val.get_instruction(self.context) {
            match instruction {
                Instruction::AddrOf(arg) => self.compile_addr_of(instr_val, arg),
                Instruction::AsmBlock(asm, args) => {
                    check!(
                        self.compile_asm_block(instr_val, asm, args),
                        return err(warnings, errors),
                        warnings,
                        errors
                    )
                }
                Instruction::BitCast(val, ty) => self.compile_bitcast(instr_val, val, ty),
                Instruction::BinaryOp { op, arg1, arg2 } => {
                    self.compile_binary_op(instr_val, op, arg1, arg2)
                }
                Instruction::Branch(to_block) => self.compile_branch(to_block),
                Instruction::Call(func, args) => self.compile_call(instr_val, func, args),
                Instruction::CastPtr(val, ty, offs) => {
                    self.compile_cast_ptr(instr_val, val, ty, *offs)
                }
                Instruction::Cmp(pred, lhs_value, rhs_value) => {
                    self.compile_cmp(instr_val, pred, lhs_value, rhs_value)
                }
                Instruction::ConditionalBranch {
                    cond_value,
                    true_block,
                    false_block,
                } => check!(
                    self.compile_conditional_branch(cond_value, true_block, false_block),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::ContractCall {
                    params,
                    coins,
                    asset_id,
                    gas,
                    ..
                } => self.compile_contract_call(instr_val, params, coins, asset_id, gas),
                Instruction::ExtractElement {
                    array,
                    ty,
                    index_val,
                } => self.compile_extract_element(instr_val, array, ty, index_val),
                Instruction::ExtractValue {
                    aggregate, indices, ..
                } => self.compile_extract_value(instr_val, aggregate, indices),
                Instruction::FuelVm(fuel_vm_instr) => {
                    errors.push(CompileError::Internal(
                        "Invalid FuelVM IR instruction provided to the EVM code gen.",
                        self.md_mgr
                            .val_to_span(self.context, *instr_val)
                            .unwrap_or_else(Self::empty_span),
                    ));
                }
                Instruction::GetLocal(local_var) => self.compile_get_local(instr_val, local_var),
                Instruction::InsertElement {
                    array,
                    ty,
                    value,
                    index_val,
                } => self.compile_insert_element(instr_val, array, ty, value, index_val),
                Instruction::InsertValue {
                    aggregate,
                    value,
                    indices,
                    ..
                } => self.compile_insert_value(instr_val, aggregate, value, indices),
                Instruction::IntToPtr(val, _) => self.compile_int_to_ptr(instr_val, val),
                Instruction::Load(src_val) => check!(
                    self.compile_load(instr_val, src_val),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                Instruction::MemCopy {
                    dst_val,
                    src_val,
                    byte_len,
                } => self.compile_mem_copy(instr_val, dst_val, src_val, *byte_len),
                Instruction::Nop => (),
                Instruction::Ret(ret_val, ty) => {
                    if func_is_entry {
                        self.compile_ret_from_entry(instr_val, ret_val, ty)
                    } else {
                        self.compile_ret_from_call(instr_val, ret_val)
                    }
                }
                Instruction::Store {
                    dst_val,
                    stored_val,
                } => check!(
                    self.compile_store(instr_val, dst_val, stored_val),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
            }
        } else {
            errors.push(CompileError::Internal(
                "Value not an instruction.",
                self.md_mgr
                    .val_to_span(self.context, *instr_val)
                    .unwrap_or_else(Self::empty_span),
            ));
        }
        ok((), warnings, errors)
    }

    fn compile_asm_block(
        &mut self,
        instr_val: &Value,
        asm: &AsmBlock,
        asm_args: &[AsmArg],
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_addr_of(&mut self, instr_val: &Value, arg: &Value) {
        todo!();
    }

    fn compile_bitcast(&mut self, instr_val: &Value, bitcast_val: &Value, to_type: &Type) {
        todo!();
    }

    fn compile_binary_op(
        &mut self,
        instr_val: &Value,
        op: &BinaryOpKind,
        arg1: &Value,
        arg2: &Value,
    ) {
        todo!();
    }

    fn compile_branch(&mut self, to_block: &BranchToWithArgs) {
        todo!();
    }

    fn compile_cast_ptr(&mut self, instr_val: &Value, val: &Value, ty: &Type, offs: u64) {
        todo!();
    }

    fn compile_cmp(
        &mut self,
        instr_val: &Value,
        pred: &Predicate,
        lhs_value: &Value,
        rhs_value: &Value,
    ) {
        todo!();
    }

    fn compile_conditional_branch(
        &mut self,
        cond_value: &Value,
        true_block: &BranchToWithArgs,
        false_block: &BranchToWithArgs,
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_branch_to_phi_value(&mut self, to_block: &BranchToWithArgs) {
        todo!();
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_contract_call(
        &mut self,
        instr_val: &Value,
        params: &Value,
        coins: &Value,
        asset_id: &Value,
        gas: &Value,
    ) {
        todo!();
    }

    fn compile_extract_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Type,
        index_val: &Value,
    ) {
        todo!();
    }

    fn compile_extract_value(&mut self, instr_val: &Value, aggregate_val: &Value, indices: &[u64]) {
        todo!();
    }

    fn compile_get_storage_key(&mut self, instr_val: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_get_local(&mut self, instr_val: &Value, local_var: &LocalVar) {
        todo!();
    }

    fn compile_gtf(&mut self, instr_val: &Value, index: &Value, tx_field_id: u64) {
        todo!();
    }

    fn compile_insert_element(
        &mut self,
        instr_val: &Value,
        array: &Value,
        ty: &Type,
        value: &Value,
        index_val: &Value,
    ) {
        todo!();
    }

    fn compile_insert_value(
        &mut self,
        instr_val: &Value,
        aggregate_val: &Value,
        value: &Value,
        indices: &[u64],
    ) {
        todo!();
    }

    fn compile_int_to_ptr(&mut self, instr_val: &Value, int_to_ptr_val: &Value) {
        todo!();
    }

    fn compile_load(&mut self, instr_val: &Value, src_val: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_mem_copy(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        src_val: &Value,
        byte_len: u64,
    ) {
        todo!();
    }

    fn compile_log(&mut self, instr_val: &Value, log_val: &Value, log_ty: &Type, log_id: &Value) {
        todo!();
    }

    fn compile_read_register(&mut self, instr_val: &Value, reg: &sway_ir::Register) {
        todo!();
    }

    fn compile_ret_from_entry(&mut self, instr_val: &Value, ret_val: &Value, ret_type: &Type) {
        if ret_type.is_unit(self.context) {
            // Unit returns should always be zero, although because they can be omitted from
            // functions, the register is sometimes uninitialized. Manually return zero in this
            // case.
            self.cur_section
                .as_mut()
                .unwrap()
                .ops
                .push(AbstractOp::Op(Op::Return(Return)));
        } else {
            todo!();
        }
    }

    fn compile_revert(&mut self, instr_val: &Value, revert_val: &Value) {
        todo!();
    }

    fn compile_smo(
        &mut self,
        instr_val: &Value,
        recipient_and_message: &Value,
        message_size: &Value,
        output_index: &Value,
        coins: &Value,
    ) {
        todo!();
    }

    fn compile_state_access_quad_word(
        &mut self,
        instr_val: &Value,
        val: &Value,
        key: &Value,
        number_of_slots: &Value,
        access_type: StateAccessType,
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_state_load_word(&mut self, instr_val: &Value, key: &Value) -> CompileResult<()> {
        todo!();
    }

    fn compile_state_store_word(
        &mut self,
        instr_val: &Value,
        store_val: &Value,
        key: &Value,
    ) -> CompileResult<()> {
        todo!();
    }

    fn compile_store(
        &mut self,
        instr_val: &Value,
        dst_val: &Value,
        stored_val: &Value,
    ) -> CompileResult<()> {
        todo!();
    }

    pub(super) fn func_to_labels(&mut self, func: &Function) -> (Label, Label) {
        self.func_label_map.get(func).cloned().unwrap_or_else(|| {
            let labels = (self.get_label(), self.get_label());
            self.func_label_map.insert(*func, labels);
            labels
        })
    }

    pub fn compile_function(&mut self, function: Function) -> CompileResult<()> {
        self.cur_section = Some(EvmAsmSection::new());

        // push1 0x80
        // push1 0x40
        // mstore
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x80.into()),
            )))));
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x40.into()),
            )))));
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::MStore(MStore)));

        //self.init_locals(function);
        let func_is_entry = function.is_entry(self.context);

        // Compile instructions.
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        for block in function.block_iter(self.context) {
            self.insert_block_label(block);
            for instr_val in block.instruction_iter(self.context) {
                check!(
                    self.compile_instruction(&instr_val, func_is_entry),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
            }
        }

        // push1 0x00
        // dup1
        // revert
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::Push1(Push1(Imm::with_expression(
                Expression::Terminal(0x00.into()),
            )))));
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::Dup1(Dup1)));
        self.cur_section
            .as_mut()
            .unwrap()
            .ops
            .push(AbstractOp::new(Op::Revert(Revert)));

        // Generate the ABI.
        #[allow(deprecated)]
        self.cur_section
            .as_mut()
            .unwrap()
            .abi
            .push(ethabi::operation::Operation::Function(ethabi::Function {
                name: function.get_name(self.context).to_string(),
                inputs: vec![],
                outputs: vec![],
                constant: None,
                state_mutability: ethabi::StateMutability::NonPayable,
            }));

        self.sections.push(self.cur_section.take().unwrap());
        self.cur_section = None;

        ok((), vec![], vec![])
    }

    pub(super) fn compile_call(&mut self, instr_val: &Value, function: &Function, args: &[Value]) {
        todo!();
    }

    pub(super) fn compile_ret_from_call(&mut self, instr_val: &Value, ret_val: &Value) {
        todo!();
    }

    pub(super) fn insert_block_label(&mut self, block: Block) {
        if &block.get_label(self.context) != "entry" {
            let label = self.block_to_label(&block);
            self.cur_section
                .as_mut()
                .unwrap()
                .ops
                .push(AbstractOp::Label(label.to_string()));
        }
    }

    fn block_to_label(&mut self, block: &Block) -> Label {
        self.block_label_map.get(block).cloned().unwrap_or_else(|| {
            let label = self.get_label();
            self.block_label_map.insert(*block, label);
            label
        })
    }
}
