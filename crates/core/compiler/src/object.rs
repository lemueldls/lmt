use std::fs::File;

use cranelift::prelude::*;
use cranelift_module::{DataDescription, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use lmt_parser::{Input, Parser};
use lmt_synthesis::ModuleSynthesis;

// use serde::{Deserialize, Serialize, Serializer};
use crate::util::declare_variables;
use crate::{
    frontend::{Expr, parser},
    translator::FunctionTranslator,
};

/// The basic JIT class.
pub struct JITObject {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,

    /// The data context, which is to data objects what `ctx` is to functions.
    data_description: DataDescription,

    /// The module, with the simplejit backend, which manages the JIT'd
    /// functions.
    module: ObjectModule,
}

impl JITObject {
    /// Create a new `JIT` instance.
    pub fn new(name: &str) -> Self {
        // Windows calling conventions are not supported yet.
        if cfg!(windows) {
            unimplemented!();
        }

        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let builder = ObjectBuilder::new(
            isa,
            name.to_owned(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let module = ObjectModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,
        }
    }

    /// Compile a string in the toy language into machine code.
    pub fn compile(&mut self, input: &str) -> Result<(), String> {
        // First, parse the string, producing AST nodes.
        let (name, params, the_return, stmts) =
            parser::function(input).map_err(|e| e.to_string())?;

        // Then, translate the AST nodes into Cranelift IR.
        self.translate(params, the_return, stmts)
            .map_err(|e| e.to_string())?;

        // Next, declare the function to simple jit. Functions must be declared
        // before they can be called, or defined.
        //
        // TODO: This may be an area where the API should be streamlined; should
        // we have a version of `declare_function` that automatically declares
        // the function?
        let id = self
            .module
            .declare_function(&name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        // Define the function to simplejit. This finishes compilation, although
        // there may be outstanding relocations to perform. Currently, simplejit
        // cannot finish relocations until all functions to be called are
        // defined. For this toy demo for now, we'll just finalize the function
        // below.
        self.module
            .define_function(id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Now that compilation is finished, we can clear out the context state.
        self.module.clear_context(&mut self.ctx);

        Ok(())
    }

    /// Create a zero-initialized data section.
    pub fn create_data(&mut self, name: &str, contents: Vec<u8>) -> Result<(), String> {
        // The steps here are analogous to `compile`, except that data is much
        // simpler than functions.
        self.data_description.define(contents.into_boxed_slice());
        let id = self
            .module
            .declare_data(name, Linkage::Export, true, false)
            .map_err(|e| e.to_string())?;

        self.module
            .define_data(id, &self.data_description)
            .map_err(|e| e.to_string())?;
        self.data_description.clear();
        // self.module.finalize_definitions();
        Ok(())
    }

    // Translate from toy-language AST nodes into Cranelift IR.
    fn translate(
        &mut self,
        params: Vec<String>,
        the_return: String,
        stmts: Vec<Expr>,
    ) -> Result<(), String> {
        // Our toy language currently only supports I64 values, though Cranelift
        // supports other types.
        let int = self.module.target_config().pointer_type();

        for _p in &params {
            self.ctx.func.signature.params.push(AbiParam::new(int));
        }

        // Our toy language currently only supports one return value, though
        // Cranelift is designed to support more.
        self.ctx.func.signature.returns.push(AbiParam::new(int));

        // Create the builder to builder a function.
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create the entry block, to start emitting code in.
        let entry_block = builder.create_block();

        // Since this is the entry block, add block parameters corresponding to
        // the function's parameters.
        //
        // TODO: Streamline the API here.
        builder.append_block_params_for_function_params(entry_block);

        // Tell the builder to emit code in this block.
        builder.switch_to_block(entry_block);

        // And, tell the builder that this block will have no further
        // predecessors. Since it's the entry block, it won't have any
        // predecessors.
        builder.seal_block(entry_block);

        // The toy language allows variables to be declared implicitly.
        // Walk the AST and declare all implicitly-declared variables.
        let variables =
            declare_variables(int, &mut builder, &params, &the_return, &stmts, entry_block);

        // Now translate the statements of the function body.
        let mut trans = FunctionTranslator {
            int,
            builder,
            variables,
            data_description: &mut self.data_description,
            module: &mut self.module,
        };
        for expr in stmts {
            trans.translate_expr(expr);
        }

        // Set up the return variable of the function. Above, we declared a
        // variable to hold the return value. Here, we just do a use of that
        // variable.
        let return_variable = trans.variables.get(&the_return).unwrap();
        let return_value = trans.builder.use_var(*return_variable);

        // Emit the return instruction.
        trans.builder.ins().return_(&[return_value]);

        dbg!(&trans.builder.func);

        // Tell the builder we're done with this function.
        trans.builder.finalize();
        Ok(())
    }

    /// Consume self and write out an object file.
    pub fn finish(self) {
        let product = self.module.finish();
        let mut file = File::create("lmt.o").expect("error opening file");
        let bytes = product.emit().unwrap();
        use std::io::Write;
        file.write_all(&bytes).expect("error writing file");
    }
}
