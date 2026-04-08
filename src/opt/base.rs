use ron::{Options, Serializer, ser::PrettyConfig};
use serde::Serialize;

use crate::{
    Action,
    backend::CompilerOptions,
    opt::{
        OptAction, ValueAction,
        arena::{Arena, ArenaItem, ArenaRef},
    },
};
use std::{
    fs,
    ops::{Deref, DerefMut, Index, Range},
    sync::mpsc::{Receiver, Sender, channel},
    vec::Drain,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueInsn {
    Output,
    Input,
    AddValue(i64),
    SetValue(i64),
    BulkPrint(i64),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum BfInsn {
    #[default]
    Noop,
    Value(ValueInsn),
    OffsetValue(ValueInsn, i64),
    MovePtr(i64),
    SetAndMove(i64, i64),
    AddAndMove(i64, i64),
    CopyLoop(ArenaItem<Vec<(i64, i64)>>),
    Loop(ArenaItem<InsnBuf>),

    /// 0 = how many cells to skip while scanning
    Scan(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct InsnBuf {
    insns: Vec<BfInsn>,
}

impl InsnBuf {
    pub fn new() -> Self {
        Self { insns: Vec::new() }
    }

    pub fn add_all<I: IntoIterator<Item = BfInsn>>(&mut self, insns: I) {
        self.insns.extend(insns);
    }

    pub fn add_all_ref<'a, I: IntoIterator<Item = &'a BfInsn>>(&mut self, insns: I) {
        self.insns.extend(insns);
    }

    pub fn add(&mut self, insn: BfInsn) {
        if insn == BfInsn::Noop {
            return;
        }

        self.insns.push(insn);
    }

    pub fn remove(&mut self, idx: usize) -> BfInsn {
        self.insns.remove(idx)
    }

    pub fn remove_range(&mut self, range: Range<usize>) -> Drain<'_, BfInsn> {
        self.insns.drain(range)
    }

    pub fn indices(&self) -> Range<usize> {
        0..self.insns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.insns.is_empty()
    }

    pub fn len(&self) -> usize {
        self.insns.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, BfInsn> {
        self.insns.iter()
    }

    pub fn iter_mut(&mut self) -> InsnIter<'_> {
        InsnIter { buf: self, idx: 0 }
    }

    pub fn clear(&mut self) {
        self.insns.clear();
    }
}

impl Deref for InsnBuf {
    type Target = [BfInsn];

    fn deref(&self) -> &Self::Target {
        &self.insns
    }
}

impl DerefMut for InsnBuf {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.insns
    }
}

pub struct InsnIter<'a> {
    buf: &'a mut InsnBuf,
    idx: usize,
}

impl<'a> InsnIter<'a> {
    pub fn has_next(&self) -> bool {
        self.len() != 0 && self.idx <= self.buf.len() - 1
    }

    pub fn remove_prev(&mut self) {
        self.buf.remove(self.idx - 1);
        self.idx -= 1;
    }

    pub fn remove_prev_silent(&mut self) {
        self.buf.remove(self.idx - 1);
    }

    pub fn insert_new(&mut self, insn: BfInsn) {
        // Auto-skip noops

        if insn == BfInsn::Noop {
            return;
        }

        self.buf.insns.insert(self.idx - 1, insn);
        self.idx += 1;
    }

    pub fn replace_prev(&mut self, insn: BfInsn) {
        if insn == BfInsn::Noop {
            return self.remove_prev();
        }

        self.buf.insns[self.idx - 1] = insn;
    }

    pub fn insert(&mut self, insn: BfInsn) {
        if insn == BfInsn::Noop {
            return;
        }

        self.buf.insns.insert(self.idx, insn);
        self.idx += 1;
    }

    pub fn swap(&mut self, range: Range<usize>, new: Vec<BfInsn>) {
        self.buf.insns.splice(range, new);
    }

    pub fn insert_buf(&mut self, new: impl ExactSizeIterator<Item = BfInsn>) {
        let size = new.len();
        let x = self.idx - 1;

        self.buf.insns.splice(x..x, new);
        self.idx += size;
    }

    pub fn pos(&self) -> usize {
        self.idx
    }

    pub fn skip(&mut self, amount: usize) {
        self.idx += amount;
    }
}

impl<'a> Deref for InsnIter<'a> {
    type Target = InsnBuf;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<'a> DerefMut for InsnIter<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<'a> Index<usize> for InsnIter<'a> {
    type Output = BfInsn;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf.insns[index]
    }
}

impl<'a> Iterator for InsnIter<'a> {
    type Item = (BfInsn, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.has_next() {
            None
        } else {
            let idx = self.idx;

            self.idx += 1;

            Some((self.buf[idx], idx))
        }
    }
}

pub struct OptCx {
    pub(super) loop_arena: ArenaRef<InsnBuf>,
    pub(super) copy_loop_arena: ArenaRef<Vec<(i64, i64)>>,
    pub(super) insns: InsnBuf,
    pub(super) opts: CompilerOptions,
}

pub struct ActiveOptCx<'a> {
    pub loop_arena: ArenaRef<InsnBuf>,
    pub copy_loop_arena: ArenaRef<Vec<(i64, i64)>>,
    pub insns: InsnIter<'a>,
    pub opts: CompilerOptions,

    loop_tx: Option<Sender<ArenaItem<InsnBuf>>>,
    loop_rx: Option<Receiver<ArenaItem<InsnBuf>>>,
}

impl<'a> ActiveOptCx<'a> {
    fn done(&mut self) {
        let tx = self.loop_tx.take();

        if let Some(tx) = tx {
            drop(tx);
        }

        if let Some(rx) = &self.loop_rx {
            while let Ok(item) = rx.recv() {
                self.loop_arena.drop_item(item);
            }
        }
    }

    pub fn drop_loop(&self, item: ArenaItem<InsnBuf>) {
        self.loop_tx
            .as_ref()
            .expect("Sender was dropped!")
            .send(item)
            .unwrap();
    }
}

impl<'a> Drop for ActiveOptCx<'a> {
    fn drop(&mut self) {
        self.done();
    }
}

impl OptCx {
    pub fn new(opts: CompilerOptions) -> Self {
        Self {
            loop_arena: Arena::new(),
            copy_loop_arena: Arena::new(),
            insns: InsnBuf::new(),
            opts,
        }
    }

    pub fn accept(mut self, actions: &Vec<Action>) -> Self {
        self.insns = self.convert(actions);
        self
    }

    pub fn activate(&mut self) -> ActiveOptCx<'_> {
        let (loop_tx, loop_rx) = channel();

        ActiveOptCx {
            loop_arena: self.loop_arena.clone(),
            copy_loop_arena: self.copy_loop_arena.clone(),
            insns: self.insns.iter_mut(),
            opts: self.opts.clone(),
            loop_tx: Some(loop_tx),
            loop_rx: Some(loop_rx),
        }
    }

    pub fn optimize_loops<F: Fn(ActiveOptCx)>(&mut self, func: F) {
        let (loop_tx, loop_rx) = channel();

        // FIXME: This clone is expensive for large arrays.
        let mut loops = self.loop_arena.all().clone();

        for item in loops.iter_mut() {
            let active = ActiveOptCx {
                loop_arena: self.loop_arena.clone(),
                copy_loop_arena: self.copy_loop_arena.clone(),
                insns: item.iter_mut(),
                opts: self.opts.clone(),
                loop_tx: Some(loop_tx.clone()),
                loop_rx: None,
            };

            func(active);
        }

        *self.loop_arena.all_mut() = loops;
        drop(loop_tx);

        while let Ok(item) = loop_rx.recv() {
            self.loop_arena.drop_item(item);
        }
    }

    pub fn optimize<F: Fn(ActiveOptCx)>(&mut self, func: F) {
        func(self.activate());
        self.optimize_loops(func);
    }

    pub fn finish(self) -> Vec<OptAction> {
        let res = self.convert_v1(&self.insns);

        if let Some(path) = &self.opts.output_tokens {
            log::debug!("Serializing tokens...");

            let mut out = String::new();

            let mut ser = Serializer::with_options(
                &mut out,
                Some(PrettyConfig::new()),
                &Options::default().without_recursion_limit(),
            )
            .unwrap();

            let ser = serde_stacker::Serializer::new(&mut ser);

            res.serialize(ser).unwrap();
            fs::write(path, out).unwrap();

            log::debug!("Token dump written!");
        }

        res
    }

    fn convert_v1(&self, insns: &InsnBuf) -> Vec<OptAction> {
        let mut buf = Vec::new();

        for insn in insns.iter() {
            buf.push(match insn {
                BfInsn::Noop => OptAction::Noop,
                BfInsn::Value(it) => OptAction::Value(self.convert_value_v1(it)),

                BfInsn::OffsetValue(it, offs) => {
                    OptAction::OffsetValue(self.convert_value_v1(it), *offs)
                }

                BfInsn::MovePtr(val) => OptAction::MovePtr(*val),
                BfInsn::SetAndMove(set, val) => OptAction::SetAndMove(*set, *val),
                BfInsn::AddAndMove(add, val) => OptAction::AddAndMove(*add, *val),
                BfInsn::Scan(skip) => OptAction::Scan(*skip),

                BfInsn::CopyLoop(it) => {
                    OptAction::CopyLoop(self.copy_loop_arena.fetch(*it).clone())
                }

                BfInsn::Loop(it) => {
                    // clone so the read guard is dropped
                    let inner = self.loop_arena.fetch(*it).clone();

                    OptAction::Loop(self.convert_v1(&inner))
                }
            });
        }

        buf
    }

    fn convert_value_v1(&self, it: &ValueInsn) -> ValueAction {
        match it {
            ValueInsn::Output => ValueAction::Output,
            ValueInsn::Input => ValueAction::Input,
            ValueInsn::AddValue(add) => ValueAction::AddValue(*add),
            ValueInsn::SetValue(set) => ValueAction::SetValue(*set),
            ValueInsn::BulkPrint(num) => ValueAction::BulkPrint(*num),
        }
    }

    fn convert(&self, actions: &Vec<Action>) -> InsnBuf {
        let mut buf = InsnBuf::new();

        for action in actions {
            match action {
                Action::Right => buf.add(BfInsn::MovePtr(1)),
                Action::Left => buf.add(BfInsn::MovePtr(-1)),
                Action::Inc => buf.add(BfInsn::Value(ValueInsn::AddValue(1))),
                Action::Dec => buf.add(BfInsn::Value(ValueInsn::AddValue(-1))),
                Action::Output => buf.add(BfInsn::Value(ValueInsn::Output)),
                Action::Input => buf.add(BfInsn::Value(ValueInsn::Input)),

                Action::Loop(actions) => {
                    let conv = self.convert(actions);
                    let item = self.loop_arena.alloc();

                    *self.loop_arena.fetch_mut(item) = conv;
                    buf.add(BfInsn::Loop(item));
                }
            }
        }

        buf
    }
}
