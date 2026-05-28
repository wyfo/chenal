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
	mov r10, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
.LBB9_1:
	mov rax, r15
	jmp .LBB9_2
.LBB9_7:
	mov rax, r15
	lock cmpxchg	qword ptr [r14], r8
	je .LBB9_8
.LBB9_2:
	mov r15, rax
	mov rdi, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rsi, rax
	and rsi, r15
	dec rdi
	cmp rsi, rdi
	seta dil
	sbb dil, 0
	je .LBB9_5
	movzx eax, dil
	cmp eax, 255
	jne .LBB9_15
	lea r8, [r15 + 1]
	jmp .LBB9_6
.LBB9_5:
	or eax, r15d
	inc eax
	mov r8, r15
	and r8, rbp
	or r8, rax
.LBB9_6:
	mov edi, r15d
	mov rax, r15
	shr rax, 32
	cmp eax, r15d
	jne .LBB9_7
	mov rax, qword ptr [r14]
	cmp rax, r15
	jne .LBB9_2
	mov rax, qword ptr [r14 + 128]
	mov r9d, dword ptr [r14 + 416]
	add eax, r9d
	inc eax
	cmp rax, rdi
	je .LBB9_20
	mov r8d, r8d
	shl rax, 32
	or rax, r8
	mov r8, rax
	jmp .LBB9_7
.LBB9_20:
	test dl, 1
	je .LBB9_21
	mov r13, rcx
	mov rdi, rcx
	mov r12, r10
	call r10
	xor edx, edx
	cmp al, 2
	mov rcx, r13
	mov r10, r12
	je .LBB9_1
	jmp .LBB9_30
.LBB9_21:
	mov r12, r10
	mov r13, rcx
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_23
	lea rax, [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r14 + 256]
	mov qword ptr [rsp + 8], rax
	mov byte ptr [rsp + 48], 2
.LBB9_23:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 56], rax
	mov qword ptr [rsp + 64], rax
	mov qword ptr [rsp + 72], 0
	lea rdi, [rsp + 8]
	lea rsi, [rsp + 56]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov dl, 1
	test al, al
	mov rcx, r13
	mov r10, r12
	jne .LBB9_1
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_26
.LBB9_27:
	mov qword ptr [rsp + 8], 0
	xor edx, edx
	jmp .LBB9_1
.LBB9_26:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov r10, r12
	mov rcx, r13
	jmp .LBB9_27
.LBB9_8:
	shl rsi, 4
	add rsi, qword ptr [r14 + 400]
	mov qword ptr [rbx], rsi
	mov qword ptr [rbx + 8], rdi
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_18
.LBB9_19:
	add rsp, 88
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_15:
	mov byte ptr [rbx + 8], 0
	mov qword ptr [rbx], 0
	cmp qword ptr [rsp + 8], 0
	je .LBB9_19
.LBB9_18:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_19
.LBB9_30:
	mov byte ptr [rbx + 8], al
	mov qword ptr [rbx], 0
	cmp qword ptr [rsp + 8], 0
	je .LBB9_19
	jmp .LBB9_18
	mov rbx, rax
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_13
.LBB9_14:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB9_13:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_14
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
