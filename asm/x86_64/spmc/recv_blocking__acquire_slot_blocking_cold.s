chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 88
	mov r15, rdx
	mov r14, rsi
	mov rbx, rdi
	mov qword ptr [rsp + 8], 0
	xor edx, edx
	movabs rbp, -4294967296
	mov r11, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
.LBB9_1:
	mov rax, r15
	jmp .LBB9_2
.LBB9_9:
	mov rax, qword ptr [r14]
	mov r9, qword ptr [r14 + 424]
	test r9, r9
	je .LBB9_10
	mov rax, qword ptr [r14]
	mov r9d, eax
	lea r10, [4*r9 + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 424], r10
	sete r10b
	shr rax, 2
	test r10b, r10b
	cmovne rax, r9
	cmp r8, rax
	je .LBB9_17
.LBB9_13:
	mov edi, edi
	shl rax, 32
	or rax, rdi
	mov rdi, rax
.LBB9_6:
	mov rax, qword ptr [r14 + 400]
	mov rsi, qword ptr [rax + 8*rsi]
	#MEMBARRIER
	mov rax, r15
	lock cmpxchg	qword ptr [r14 + 128], rdi
	je .LBB9_7
.LBB9_2:
	mov r15, rax
	mov rdi, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rsi, rax
	and rsi, r15
	dec rdi
	cmp rsi, rdi
	jne .LBB9_4
	or eax, r15d
	inc eax
	mov rdi, r15
	and rdi, rbp
	or rdi, rax
	jmp .LBB9_5
.LBB9_4:
	lea rdi, [r15 + 1]
.LBB9_5:
	mov r8d, r15d
	mov rax, r15
	shr rax, 32
	cmp r8d, eax
	jne .LBB9_6
	mov rax, qword ptr [r14 + 128]
	cmp rax, r15
	jne .LBB9_2
	jmp .LBB9_9
.LBB9_10:
	cmp r8d, eax
	je .LBB9_22
	mov eax, eax
	jmp .LBB9_13
.LBB9_22:
	test dl, 1
	je .LBB9_23
	mov r13, rcx
	mov rdi, rcx
	mov r12, r11
	call r11
	xor edx, edx
	cmp al, 2
	mov rcx, r13
	mov r11, r12
	je .LBB9_1
	jmp .LBB9_29
.LBB9_23:
	mov r13, r11
	mov qword ptr [rsp], rcx
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_25
	lea rax, [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r14 + 256]
	mov qword ptr [rsp + 8], rax
	mov byte ptr [rsp + 48], 2
.LBB9_25:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 56], rax
	mov qword ptr [rsp + 64], rax
	mov qword ptr [rsp + 72], 0
	lea rdi, [rsp + 8]
	lea rsi, [rsp + 56]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov edx, eax
	mov rcx, qword ptr [rsp]
	mov r11, r13
	jmp .LBB9_1
.LBB9_7:
	mov qword ptr [rbx + 8], rsi
	xor eax, eax
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_20
.LBB9_21:
	add rsp, 88
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_17:
	mov byte ptr [rbx + 1], 0
.LBB9_18:
	mov al, 1
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	je .LBB9_21
.LBB9_20:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_21
.LBB9_29:
	mov byte ptr [rbx + 1], al
	jmp .LBB9_18
	mov rbx, rax
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_14
.LBB9_16:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB9_14:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_16
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
