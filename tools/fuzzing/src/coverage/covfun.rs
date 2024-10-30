use super::util::{cursor_align, read_int, Leb128};
use std::io::Cursor;

//#[derive(Debug)]
pub struct Record {
    pub name_hash: u64,
    pub data_len: u32,
    pub func_hash: u64,
    pub translation_unit_hash: u64,
}

impl Record {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Self {
            name_hash: read_int(cursor),
            data_len: read_int(cursor),
            func_hash: read_int(cursor),
            translation_unit_hash: read_int(cursor),
        }
    }
}

//#[derive(Debug)]
pub enum PseudoCounter {
    ExpansionRegion(u64),
    CodeRegion,
    SkippedRegion,
    BranchRegion,
}

impl PseudoCounter {
    #[coverage(off)]
    pub fn new(value: u64) -> Self {
        let data = value >> 1;

        match value & 1 {
            0 => match data {
                0 => Self::CodeRegion,
                2 => Self::SkippedRegion,
                4 => Self::BranchRegion,
                _ => panic!(),
            },
            _ => Self::ExpansionRegion(data),
        }
    }
}

//#[derive(Debug)]
pub enum Counter {
    Zero,
    Reference(usize),
    Substraction(usize),
    Addition(usize),
}

impl Counter {
    #[coverage(off)]
    pub fn new(value: u64) -> Self {
        let idx = (value >> 2) as usize;

        match value & 0b11 {
            0 => Self::Zero,
            1 => Self::Reference(idx),
            2 => Self::Substraction(idx),
            3 => Self::Addition(idx),
            _ => unreachable!(),
        }
    }

    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Counter::new(u64::read_leb128(cursor))
    }
}

//#[derive(Debug)]
pub enum Header {
    PseudoCounter(PseudoCounter),
    Counter(Counter),
}

impl Header {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        let header = u64::read_leb128(cursor);
        let data = header >> 2;

        match header & 0b11 {
            0 => Self::PseudoCounter(PseudoCounter::new(data)),
            _ => Self::Counter(Counter::new(header)),
        }
    }
}

//#[derive(Debug)]
pub struct CounterExpression {
    pub lhs: Counter,
    pub rhs: Counter,
}

impl CounterExpression {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Self {
            lhs: Counter::read(cursor),
            rhs: Counter::read(cursor),
        }
    }

    #[coverage(off)]
    fn resolve_op(op: &Counter, counters: &'static [i64], expressions: &Vec<Self>) -> i64 {
        match op {
            Counter::Zero => 0,
            Counter::Reference(idx) => counters[*idx],
            Counter::Substraction(idx) => expressions[*idx].resolve_sub(counters, expressions),
            Counter::Addition(idx) => expressions[*idx].resolve_add(counters, expressions),
        }
    }

    #[coverage(off)]
    pub fn resolve_sub(&self, counters: &'static [i64], expressions: &Vec<Self>) -> i64 {
        let lhs = Self::resolve_op(&self.lhs, counters, expressions);
        let rhs = Self::resolve_op(&self.rhs, counters, expressions);
        //println!("sub {} {}", lhs, rhs);
        lhs.wrapping_sub(rhs)
    }

    #[coverage(off)]
    pub fn resolve_add(&self, counters: &'static [i64], expressions: &Vec<Self>) -> i64 {
        let lhs = Self::resolve_op(&self.lhs, counters, expressions);
        let rhs = Self::resolve_op(&self.rhs, counters, expressions);
        //println!("add {} {}", lhs, rhs);
        lhs.wrapping_add(rhs)
    }
}

//#[derive(Debug, Clone)]
pub struct SourceRange {
    pub delta_line_start: usize,
    pub column_start: usize,
    pub num_lines: usize,
    pub column_end: usize,
}

impl Clone for SourceRange {
    #[coverage(off)]
    fn clone(&self) -> Self {
        Self {
            delta_line_start: self.delta_line_start,
            column_start: self.column_start,
            num_lines: self.num_lines,
            column_end: self.column_end,
        }
    }
}

impl SourceRange {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Self {
            delta_line_start: usize::read_leb128(cursor),
            column_start: usize::read_leb128(cursor),
            num_lines: usize::read_leb128(cursor),
            column_end: usize::read_leb128(cursor),
        }
    }
}

//#[derive(Debug)]
pub struct Region {
    pub header: Header,
    pub source_range: SourceRange,
}

impl Region {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        Self {
            header: Header::read(cursor),
            source_range: SourceRange::read(cursor),
        }
    }
}

//#[derive(Debug)]
pub struct FunCov {
    pub function_record: Record,
    pub expressions: Vec<CounterExpression>,
    pub mapping_regions: Vec<(u64, Vec<Region>)>,
}

impl FunCov {
    #[coverage(off)]
    pub fn read(cursor: &mut Cursor<&Vec<u8>>) -> Self {
        let function_record = Record::read(cursor);
        //let data_pos = cursor.position();

        assert!(function_record.data_len > 0);
        // numIndices : LEB128, filenameIndex0 : LEB128, filenameIndex1 : LEB128...
        let num_indices = u64::read_leb128(cursor);
        let file_id_mapping: Vec<u64> = (0..num_indices)
            .map(
                #[coverage(off)]
                |_| u64::read_leb128(cursor),
            )
            .collect();

        // numExpressions : LEB128, expr0LHS : LEB128, expr0RHS : LEB128, expr1LHS : LEB128, expr1RHS : LEB128...
        let num_expressions = u64::read_leb128(cursor);
        let expressions: Vec<CounterExpression> = (0..num_expressions)
            .map(
                #[coverage(off)]
                |_| CounterExpression::read(cursor),
            )
            .collect();

        // [numRegionArrays : LEB128, regionsForFile0, regionsForFile1, ...]
        // Not actually included?
        // let num_region_arrays = leb128::read::unsigned(&mut covfun_cursor).unwrap();

        let mapping_regions: Vec<(u64, Vec<Region>)> = file_id_mapping
            .iter()
            .map(
                #[coverage(off)]
                |index| {
                    // [numRegions : LEB128, region0, region1, ...]
                    let num_regions = u64::read_leb128(cursor);
                    let regions: Vec<Region> = (0..num_regions)
                        .map(
                            #[coverage(off)]
                            |_| Region::read(cursor),
                        )
                        .collect();
                    (*index, regions)
                },
            )
            .collect();

        cursor_align::<u64>(cursor);

        Self {
            function_record,
            expressions,
            mapping_regions,
        }
    }
}
