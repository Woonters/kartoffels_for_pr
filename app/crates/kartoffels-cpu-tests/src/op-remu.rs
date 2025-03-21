#![cfg_attr(target_arch = "riscv32", no_std, no_main)]

kartoffels_cpu_tests::test! {
    r#"
    .global _start
    .attribute arch, "rv32im"

    _start:
        li x1, -100
        li x2, 23
        remu x3, x1, x2
        remu x4, x2, x0
        ebreak
    "#
}

/*
 * x1 = -100
 * x2 = 23
 * x3 = 4
 * x4 = -1
 */
