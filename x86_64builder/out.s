.intel_syntax noprefix
.global main
.section .data
hello_world:
    .asciz "Hello, World!"
.section .text
main:
    push rbp
    mov rbp, rsp
    lea rdi, [rip+hello_world]
    call puts
    mov rax, 0
    leave
    ret
