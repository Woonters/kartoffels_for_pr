#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start

    _start:
        li x1, 123
        srai x2, x1, 4
        srai x3, x1, 31
        ebreak
    "#
}

/*
 * x1 = 123
 * x2 = 7
 * x3 = 0
 */
