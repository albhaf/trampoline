.globl _trampoline

.extern _trampoline_start
.extern _trampoline_end
.extern _get_implementation

_trampoline:

	subq   $0x80+8, %rsp     // 8 x 16-bytes xmm registers + 8-bytes alignment
	movdqa %xmm0, 0x70(%rsp)
	movdqa %xmm1, 0x60(%rsp)
	movdqa %xmm2, 0x50(%rsp)
	movdqa %xmm3, 0x40(%rsp)
	movdqa %xmm4, 0x30(%rsp)
	movdqa %xmm5, 0x20(%rsp)
	movdqa %xmm6, 0x10(%rsp)
	movdqa %xmm7, 0x00(%rsp)
	pushq  %rdi
	pushq  %rsi
	pushq  %rdx
	pushq  %rcx
	pushq  %r8
	pushq  %r9
	pushq  %rax
	pushq  %r12

	movq %rsp, %r12
	andq $-0x10, %rsp

	callq _get_implementation

	movq %r12, %rsp

	popq  %r12
	pushq %rax
	pushq %r12

	movq %rsp, %r12
	andq $-0x10, %rsp

	// allocate 16 bytes
	movq $0x10, %rdi

	callq _malloc

	// store the initial value of calle saved registers %r13 and %r14
	movq %r13, 0x0(%rax)
	movq %r14, 0x8(%rax)

	movq 0xd8(%rsp), %r13 // caller of RCTProfileTrampoline (0xd8 is stack top)
	movq %rax, %r14       // allocated memory's address

	movq 0x40(%r12), %rdi // object
	movq 0x38(%r12), %rsi // selector

	callq _trampoline_start

	movq %r12, %rsp
	popq %r12

	popq   %r11
	popq   %rax
	popq   %r9
	popq   %r8
	popq   %rcx
	popq   %rdx
	popq   %rsi
	popq   %rdi
	movdqa 0x00(%rsp), %xmm7
	movdqa 0x10(%rsp), %xmm6
	movdqa 0x20(%rsp), %xmm5
	movdqa 0x30(%rsp), %xmm4
	movdqa 0x40(%rsp), %xmm3
	movdqa 0x50(%rsp), %xmm2
	movdqa 0x60(%rsp), %xmm1
	movdqa 0x70(%rsp), %xmm0
	addq   $0x80+8, %rsp

	addq $0x8, %rsp

	callq  *%r11
	pushq  %rax
	pushq  %rdx
	subq   $0x20, %rsp       // 2 16-bytes xmm register
	movdqa %xmm0, 0x00(%rsp)
	movdqa %xmm1, 0x10(%rsp)

	callq _trampoline_end

	movq %r13, %rcx
	movq %r14, %rdi
	movq 0x0(%r14), %r13
	movq 0x8(%r14), %r14

	pushq %rcx
	pushq %r12
	movq  %rsp, %r12
	andq  $-0x10, %rsp

	callq _free

	movq %r12, %rsp
	popq %r12

	popq   %rcx
	movdqa 0x00(%rsp), %xmm0
	movdqa 0x10(%rsp), %xmm1
	addq   $0x20, %rsp
	popq   %rdx
	popq   %rax

	// jump to caller
	jmpq *%rcx
