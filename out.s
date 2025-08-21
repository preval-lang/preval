Module {
    constants: [
        (
            Array(
                u8,
                1,
            ),
            [
                53,
            ],
        ),
    ],
    functions: {
        "main": Function {
            ir: [
                Block {
                    statements: [
                        Operation(
                            Call {
                                function: [
                                    "get_five",
                                ],
                                args: [],
                            },
                            Some(
                                0,
                            ),
                        ),
                        Operation(
                            Call {
                                function: [
                                    "print",
                                ],
                                args: [
                                    0,
                                ],
                            },
                            Some(
                                1,
                            ),
                        ),
                    ],
                    terminal: Evaluate(
                        Some(
                            1,
                        ),
                    ),
                },
            ],
            exported: true,
            variable_types: [
                Slice(
                    u8,
                ),
                void,
            ],
        },
        "get_five": Function {
            ir: [
                Block {
                    statements: [
                        Operation(
                            LoadGlobal {
                                src: 0,
                            },
                            Some(
                                0,
                            ),
                        ),
                    ],
                    terminal: Evaluate(
                        Some(
                            0,
                        ),
                    ),
                },
            ],
            exported: true,
            variable_types: [
                Slice(
                    u8,
                ),
            ],
        },
    },
}