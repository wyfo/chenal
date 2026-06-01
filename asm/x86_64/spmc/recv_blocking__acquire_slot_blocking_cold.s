chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold:
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
.LBB10_1:
	mov rax, r15
	jmp .LBB10_2
.LBB10_10:
	cmp r8d, r9d
	je .LBB10_11
.LBB10_16:
	mov eax, edi
	shl r9, 32
	or r9, rax
	mov rdi, r9
.LBB10_6:
	mov rax, qword ptr [r14 + 400]
	mov rsi, qword ptr [rax + 8*rsi]
	#MEMBARRIER
	mov rax, r15
	lock cmpxchg	qword ptr [r14 + 128], rdi
	je .LBB10_7
.LBB10_2:
	mov r15, rax
	mov rdi, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rsi, rax
	and rsi, r15
	dec rdi
	cmp rsi, rdi
	jne .LBB10_4
	or eax, r15d
	inc eax
	mov rdi, r15
	and rdi, rbp
	or rdi, rax
	jmp .LBB10_5
.LBB10_4:
	lea rdi, [r15 + 1]
.LBB10_5:
	mov r8d, r15d
	mov rax, r15
	shr rax, 32
	cmp r8d, eax
	jne .LBB10_6
	mov rax, qword ptr [r14 + 128]
	cmp rax, r15
	jne .LBB10_2
	mov rax, qword ptr [r14 + 424]
	mov r9, qword ptr [r14]
	mov r9d, r9d
	test rax, rax
	je .LBB10_10
	lea r10, [4*r9 + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 424], r10
	sete r10b
	shr rax, 2
	test r10b, r10b
	cmove r9, rax
	cmp r8, r9
	jne .LBB10_16
	jmp .LBB10_20
.LBB10_11:
	test dl, 1
	je .LBB10_25
	mov r13, rcx
	mov rdi, rcx
	mov r12, r11
	call r11
	xor edx, edx
	cmp al, 2
	mov rcx, r13
	mov r11, r12
	je .LBB10_1
	jmp .LBB10_14
.LBB10_25:
	mov r13, r11
	mov qword ptr [rsp], rcx
	cmp qword ptr [rsp + 8], 0
	jne .LBB10_27
	lea rax, [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r14 + 256]
	mov qword ptr [rsp + 8], rax
	mov byte ptr [rsp + 48], 2
.LBB10_27:
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
	jmp .LBB10_1
.LBB10_7:
	mov qword ptr [rbx + 8], rsi
	xor eax, eax
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	jne .LBB10_23
.LBB10_24:
	add rsp, 88
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB10_20:
	mov byte ptr [rbx + 1], 0
.LBB10_21:
	mov al, 1
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	je .LBB10_24
.LBB10_23:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB10_24
.LBB10_14:
	mov byte ptr [rbx + 1], al
	jmp .LBB10_21
	mov rbx, rax
	cmp qword ptr [rsp + 8], 0
	jne .LBB10_17
.LBB10_19:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB10_17:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB10_19
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
