(module
  ;; Simple main function that always returns `()`
  (func $Test_f_main (param $0 i32) (result i32)
    ;; Grow memory
    (drop
      (memory.grow (i32.const ${n}))
    )

    (local.set 
      $0
      (call $scrypto_alloc
        (i32.const 1)
      )
    )
    (i32.add
      (local.get $0)
      (i32.const 4)
    )
    (i32.const 0)
    (i32.store8)
    (local.get $0)
  )

  (memory $0 1)
  (export "memory" (memory $0))
  (export "scrypto_alloc" (func $scrypto_alloc))
  (export "scrypto_free" (func $scrypto_free))
  (export "Test_f_main" (func $Test_f_main))

  ${memcpy}
  ${buffer}
)