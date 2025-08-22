Module {
    constants: [
        (
            u8,
            [
                5,
            ],
        ),
    ],
    functions: {
        "main": Function {
            ir: [
                Block {
                    statements: [
                        Operation(
                            LoadGlobal {
                                src: 0,
                            },
                            Some(
                                1,
                            ),
                        ),
                        Operation(
                            LoadLocal {
                                src: 1,
                            },
                            Some(
                                2,
                            ),
                        ),
                        Operation(
                            LoadLocal {
                                src: 0,
                            },
                            Some(
                                3,
                            ),
                        ),
                        Operation(
                            Call {
                                function: [
                                    "print",
                                ],
                                args: [
                                    3,
                                ],
                            },
                            None,
                        ),
                    ],
                    terminal: Evaluate(
                        Some(
                            4,
                        ),
                    ),
                },
            ],
            exported: true,
            variable_types: [
                Slice(
                    u8,
                ),
                u8,
                u8,
                Slice(
                    u8,
                ),
            ],
            signature: Signature {
                args: [
                    Slice(
                        u8,
                    ),
                ],
                returns: void,
            },
        },
    },
}