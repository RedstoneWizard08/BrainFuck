use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ValueAction {
    Output,
    Input,
    AddValue(i64),
    SetValue(i64),
    BulkPrint(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum OptAction {
    Noop,
    Value(ValueAction),
    OffsetValue(ValueAction, i64),
    MovePtr(i64),
    SetAndMove(i64, i64),
    AddAndMove(i64, i64),
    CopyLoop(Vec<(i64, i64)>),
    Loop(Vec<OptAction>),

    /// 0 = how many cells to skip while scanning
    Scan(i64),
}
