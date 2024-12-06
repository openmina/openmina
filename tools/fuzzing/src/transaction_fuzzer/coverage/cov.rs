use core::slice;
use std::io::Cursor;

use super::{
    covfun::{Counter, CounterExpression, FunCov, Header, Region, SourceRange},
    covmap::CovMap,
    names::Names,
    profile_data::ProfileData,
    util::{get_counters, get_elf_sections, get_module_path},
};

//#[derive(Debug)]
pub struct Sections {
    pub covmap: Vec<CovMap>,
    pub covfun: Vec<FunCov>,
}

impl Default for Sections {
    #[coverage(off)]
    fn default() -> Self {
        let section_names = vec!["__llvm_covmap", "__llvm_covfun"];
        let sections = get_elf_sections(get_module_path(), &section_names);
        let covmap = &sections["__llvm_covmap"];
        let covmap_len = covmap.len() as u64;
        let mut covmap_cursor = Cursor::new(covmap);
        let mut covmap = Vec::new();

        while covmap_cursor.position() < covmap_len {
            covmap.push(CovMap::read(&mut covmap_cursor));
        }

        let covfun = &sections["__llvm_covfun"];
        let covfun_len = covfun.len() as u64;
        let mut covfun_cursor = Cursor::new(covfun);
        let mut covfun = Vec::new();

        while covfun_cursor.position() < covfun_len {
            covfun.push(FunCov::read(&mut covfun_cursor));
        }

        Self { covmap, covfun }
    }
}

impl Sections {
    #[coverage(off)]
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub struct FunCounters {
    pub name_hash: u64,
    pub fun_hash: u64,
    pub counters: &'static [i64],
}

#[derive(Debug, Clone)]
pub struct FileCounters {
    pub filename: String,
    pub counters: Vec<&'static [i64]>,
}

//#[derive(Debug)]
pub struct FileDump {
    pub filename: String,
    pub source_counters_vec: Vec<Vec<(i64, SourceRange)>>,
}

pub struct Cov {
    pub counters: &'static mut [i64],
    pub names: Names,
    pub data: ProfileData,
    pub sections: Sections,
}

impl Default for Cov {
    #[coverage(off)]
    fn default() -> Self {
        Self {
            counters: unsafe { get_counters() },
            names: Names::new(),
            data: ProfileData::new(),
            sections: Sections::new(),
        }
    }
}

impl Cov {
    #[coverage(off)]
    pub fn new() -> Self {
        Self::default()
    }

    #[coverage(off)]
    pub fn get_functions_counters(&mut self) -> Vec<FunCounters> {
        let mut pos = 0;
        self.data
            .0
            .iter()
            .filter_map(
                #[coverage(off)]
                |fun_control| {
                    if fun_control.func_hash == 0 {
                        None
                    } else {
                        let fun_counters = FunCounters {
                            name_hash: fun_control.name_hash,
                            fun_hash: fun_control.func_hash,
                            counters: unsafe {
                                slice::from_raw_parts(
                                    self.counters[pos as usize..].as_ptr(),
                                    fun_control.num_counters as usize,
                                )
                            },
                        };

                        assert_ne!(fun_counters.counters.len(), 0);

                        pos += fun_control.num_counters;
                        Some(fun_counters)
                    }
                },
            )
            .collect()
    }

    #[coverage(off)]
    fn for_file_cov<F>(&mut self, mut f: F)
    where
        F: FnMut(String, &'static [i64], &Vec<Region>, &Vec<CounterExpression>),
    {
        let mut fun_counters_vec = self.get_functions_counters();

        for covmap in self.sections.covmap.iter() {
            for funcov in self.sections.covfun.iter() {
                let record = &funcov.function_record;

                if record.translation_unit_hash == covmap.encoded_data_hash {
                    for (index, regions) in funcov.mapping_regions.iter() {
                        let filename = covmap.filenames.0[*index as usize].clone();

                        let pos = fun_counters_vec.iter().position(
                            #[coverage(off)]
                            |fc| {
                                fc.name_hash == record.name_hash && fc.fun_hash == record.func_hash
                            },
                        );

                        if pos.is_none() {
                            continue;
                        }

                        let fun_counters = fun_counters_vec.swap_remove(pos.unwrap());

                        f(
                            filename,
                            fun_counters.counters,
                            regions,
                            &funcov.expressions,
                        )
                    }
                }
            }
        }
    }

    #[coverage(off)]
    pub fn get_file_counters(&mut self) -> Vec<FileCounters> {
        let mut result: Vec<FileCounters> = Vec::new();

        self.for_file_cov(
            #[coverage(off)]
            |filename, counters, _, _| match result.iter_mut().find(
                #[coverage(off)]
                |c| c.filename == filename,
            ) {
                Some(existing_counters) => {
                    existing_counters.counters.push(counters);
                }
                None => {
                    let file_counters = FileCounters {
                        filename,
                        counters: vec![counters],
                    };

                    result.push(file_counters);
                }
            },
        );

        result
    }

    #[coverage(off)]
    pub fn dump(&mut self) -> Vec<FileDump> {
        let mut result: Vec<FileDump> = Vec::new();

        self.for_file_cov(
            #[coverage(off)]
            |filename, counters, regions, expressions| {
                let mut source_counters: Vec<(i64, SourceRange)> = Vec::new();

                for region in regions.iter() {
                    let counter = match &region.header {
                        Header::Counter(counter) => match counter {
                            Counter::Zero => 0,
                            Counter::Reference(idx) => counters[*idx],
                            Counter::Substraction(idx) => {
                                let expr = &expressions[*idx];
                                expr.resolve_sub(counters, expressions)
                            }
                            Counter::Addition(idx) => {
                                let expr = &expressions[*idx];
                                expr.resolve_add(counters, expressions)
                            }
                        },
                        Header::PseudoCounter(_pseudo_counter) => continue, // TODO
                    };

                    /*
                        FIXME?: in some cases I observe negative values

                        SubExpr => CounterExpression { lhs: Reference(0), rhs: Reference(1) }
                        counter => -1632
                    */
                    //println!("counter => {:?}", counter);
                    source_counters.push((counter, region.source_range.clone()));
                }

                match result.iter_mut().find(
                    #[coverage(off)]
                    |c| c.filename == filename,
                ) {
                    Some(existing_file_dump) => {
                        existing_file_dump.source_counters_vec.push(source_counters);
                    }
                    None => {
                        result.push(FileDump {
                            filename,
                            source_counters_vec: vec![source_counters],
                        });
                    }
                }
            },
        );

        result
    }
}
