chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 72
	mov r13, rcx
	mov qword ptr [rsp + 16], rsi
	mov r12, rdi
	lea rax, [rdx + 8]
	mov qword ptr [rsp + 8], rax
	mov rbx, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
	xor r15d, r15d
	mov qword ptr [rsp + 24], rdx
.LBB8_1:
	mov qword ptr [rsp + 32], r15
	mov rax, r13
	jmp .LBB8_2
.LBB8_8:
	lea rcx, [r13 + 1]
	mov rax, r13
	lock cmpxchg	qword ptr [r12 + 128], rcx
	je .LBB8_10
.LBB8_2:
	mov r13, rax
	mov rbp, qword ptr [r12 + 432]
	and rbp, rax
	mov rax, qword ptr [r12 + 416]
	mov rcx, rbp
	shl rcx, 4
	lea r14, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r13
	jne .LBB8_3
.LBB8_6:
	mov rdx, qword ptr [r14]
	#MEMBARRIER
	mov rax, qword ptr [r12 + 424]
	dec rax
	cmp rbp, rax
	jne .LBB8_8
	mov rcx, qword ptr [r12 + 432]
	or rcx, r13
	inc rcx
	mov rax, r13
	lock cmpxchg	qword ptr [r12 + 128], rcx
	jne .LBB8_2
	jmp .LBB8_10
.LBB8_3:
	mov rax, qword ptr [r12 + 128]
	cmp rax, r13
	jne .LBB8_2
	mov rax, qword ptr [r12]
	mov rcx, qword ptr [r12 + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r13, rdx
	je .LBB8_18
	mov rax, qword ptr [r14 + 8]
	cmp rax, r13
	je .LBB8_6
	xor r15d, r15d
	jmp .LBB8_12
.LBB8_16:
	inc r15d
.LBB8_17:
	mov rax, qword ptr [r14 + 8]
	cmp rax, r13
	je .LBB8_6
.LBB8_12:
	cmp r15d, 6
	ja .LBB8_15
	mov eax, 1
.LBB8_14:
	pause
	mov edx, eax
	mov ecx, r15d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB8_14
	jmp .LBB8_16
.LBB8_15:
	call rbx
	cmp r15d, 11
	jb .LBB8_16
	jmp .LBB8_17
.LBB8_18:
	mov edx, eax
	cmp r13, rdx
	sete cl
	mov rax, qword ptr [rsp + 32]
	not al
	test al, cl
	je .LBB8_19
	mov r14, qword ptr [rsp + 24]
	cmp qword ptr [r14], 0
	jne .LBB8_23
	lea rax, [r12 + 304]
	mov qword ptr [r14], rax
	mov rax, qword ptr [rsp + 8]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	mov byte ptr [r14 + 40], 2
.LBB8_23:
	mov rax, qword ptr [rsp + 16]
	mov qword ptr [rsp + 40], rax
	mov qword ptr [rsp + 48], rax
	mov qword ptr [rsp + 56], 0
	mov rdi, r14
	lea rsi, [rsp + 40]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov r15d, eax
	test al, al
	jne .LBB8_1
	cmp qword ptr [r14], 0
	jne .LBB8_25
.LBB8_26:
	mov qword ptr [r14], 0
	jmp .LBB8_1
.LBB8_25:
	mov rdi, r14
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB8_26
.LBB8_10:
	xor eax, eax
.LBB8_20:
	add rsp, 72
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB8_19:
	xor eax, eax
	cmp r13, rdx
	sete al
	inc rax
	jmp .LBB8_20
	mov qword ptr [r14], 0
	mov rdi, rax
	call _Unwind_Resume@PLT
