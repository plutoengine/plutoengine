#[cfg(test)]
mod test {
    use std::error::Error;
    use wasmer::{imports, Instance, Module, Store, Value};

    #[test]
    fn main() -> Result<(), Box<dyn Error>> {
        let wasm_bytes = include_bytes!("../add.wasm");

        let mut store = Store::default();
        let module = Module::new(&store, &wasm_bytes)?;
        let import_obj = imports! {};
        let instance = Instance::new(&module, &import_obj)?;
        let add_fn = instance.exports.get_function("add")?;
        let res = add_fn.call(&[Value::I32(5), Value::I32(5)])?;
        assert_eq!(res[0], Value::I32(10));

        Ok(())
    }
}
