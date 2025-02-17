use radix_engine::vm::wasm::{InvalidMemory, PrepareError, WasmValidator};
use radix_engine_queries::typed_substate_layout::PackageDefinition;
use scrypto_unit::*;

#[test]
fn test_large_data() {
    let code = wat2wasm(&include_str!("wasm/large_data.wat"));
    let definition = single_function_package_definition("Test", "f");
    let result = WasmValidator::default().validate(&code, definition.blueprints.values());

    assert!(matches!(
        result,
        Err(PrepareError::NotInstantiatable { .. })
    ));
}

#[test]
fn test_large_memory() {
    let code = wat2wasm(&include_str!("wasm/large_memory.wat"));
    let definition = single_function_package_definition("Test", "f");
    let result = WasmValidator::default().validate(&code, definition.blueprints.values());

    assert_eq!(
        Err(PrepareError::InvalidMemory(
            InvalidMemory::MemorySizeLimitExceeded
        )),
        result
    );
}

#[test]
fn invalid_export_name_should_fail() {
    // List of some invalid names (non conforming to Rust Ident).
    let names = [
        "a b",
        "a$",
        "a!",
        "a-",
        "a\u{221A}",
        "\0",
        "a\'",
        "self",
        "crate",
        "super",
        "Self",
    ];
    // Veryfing various export names like function, global and memory section.
    let replace_tokens = ["FUNCTION_NAME", "GLOBAL_NAME", "MEMORY_NAME"];

    for token in replace_tokens {
        for name in names {
            // Arrange
            let code_str = r##"
                    (module
                        (func (export "FUNCTION_NAME") (result i32)
                            i32.const 1
                        )
                        (global (export "GLOBAL_NAME") i32 (i32.const 1))
                        (memory $0 1)
                        (export "MEMORY_NAME" (memory $0))
                    )
                    "##
            .replace(token, name.clone());
            let code = wat2wasm(code_str.as_str());

            // Act
            let result = WasmValidator::default()
                .validate(&code, PackageDefinition::default().blueprints.values());

            // Assert
            assert_eq!(
                result,
                Err(PrepareError::InvalidExportName(name.to_string()))
            );
        }
    }
}
