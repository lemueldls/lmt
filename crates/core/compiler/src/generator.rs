use std::fmt;

use walrus::{ir::*, *};

pub fn generate() -> Vec<u8> {
    let values = [
        Value::String("hello world\n".to_string()),
        // Value::String("hello world\n".to_string()),
        // Value::Int32(42),
        Value::String("Ahoy!\n".to_string()),
        // Value::String("a very long string that will be written to the file\n".to_string()),
    ];

    let config = ModuleConfig::new();
    let mut module = Module::with_config(config);

    let memory = module.memories.add_local(false, 1, None);

    module.exports.add("memory", memory);

    let fd_write_type = module.types.add(
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let (fd_write, _) = module.add_import_func("wasi_unstable", "fd_write", fd_write_type);

    let mut offset = 8;

    let functions = values
        .into_iter()
        .map(|value| {
            let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
            let mut func_body = function.func_body();

            let value = value.to_string();
            let length = value.len() as i32;

            module.data.add(
                DataKind::Active(ActiveData {
                    memory,
                    location: ActiveDataLocation::Absolute(offset as u32),
                }),
                value.into_bytes(),
            );

            func_body
                .i32_const(0)
                .i32_const(offset)
                .store(memory, StoreKind::I32 { atomic: false }, MemArg {
                    align: 4,
                    offset: 0,
                })
                .i32_const(4)
                .i32_const(length)
                .store(memory, StoreKind::I32 { atomic: false }, MemArg {
                    align: 4,
                    offset: 0,
                })
                .i32_const(1)
                .i32_const(0)
                .i32_const(1)
                .i32_const(0)
                .call(fd_write)
                .drop();

            offset += length;

            function.finish(vec![], &mut module.funcs)
        })
        .collect::<Vec<_>>();

    let mut start = FunctionBuilder::new(&mut module.types, &[], &[]);

    let mut body = start.func_body();

    for function in functions.into_iter() {
        body.call(function);
    }

    let start_id = start.finish(vec![], &mut module.funcs);
    module.exports.add("_start", start_id);

    module.emit_wasm_file("target/out.wasm").unwrap();
    module.emit_wasm()
}

enum Value {
    String(String),
    Int32(i32),
}

impl Value {
    // fn size(&self) -> u32 {
    //     match self {
    //         Self::String(value) => value.len() as u32,
    //     }
    // }

    fn align(&self) -> u32 {
        match self {
            Self::String(_) => 2,
            Self::Int32(_) => 4,
        }
    }

    fn offset(&self) -> u32 {
        match self {
            Self::String(_) => 8,
            Self::Int32(_) => 4,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(value) => write!(f, "{value}"),
            Self::Int32(value) => write!(f, "{value}"),
        }
    }
}
