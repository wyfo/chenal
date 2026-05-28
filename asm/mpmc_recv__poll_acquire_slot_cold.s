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
.LBB9_1:
	mov qword ptr [rsp + 32], r15
	mov rax, r13
	jmp .LBB9_2
.LBB9_8:
	lea rcx, [r13 + 1]
	mov rax, r13
	lock cmpxchg	qword ptr [r12 + 128], rcx
	je .LBB9_10
.LBB9_2:
	mov r13, rax
	mov rbp, qword ptr [r12 + 432]
	and rbp, rax
	mov rax, qword ptr [r12 + 416]
	mov rcx, rbp
	shl rcx, 4
	lea r14, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r13
	jne .LBB9_3
.LBB9_6:
	mov rdx, qword ptr [r14]
	#MEMBARRIER
	mov rax, qword ptr [r12 + 424]
	dec rax
	cmp rbp, rax
	jne .LBB9_8
	mov rcx, qword ptr [r12 + 432]
	or rcx, r13
	inc rcx
	mov rax, r13
	lock cmpxchg	qword ptr [r12 + 128], rcx
	jne .LBB9_2
	jmp .LBB9_10
.LBB9_3:
	mov rax, qword ptr [r12 + 128]
	cmp rax, r13
	jne .LBB9_2
	mov rax, qword ptr [r12]
	mov rcx, qword ptr [r12 + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r13, rdx
	je .LBB9_18
	mov rax, qword ptr [r14 + 8]
	cmp rax, r13
	je .LBB9_6
	xor r15d, r15d
	jmp .LBB9_12
.LBB9_16:
	inc r15d
.LBB9_17:
	mov rax, qword ptr [r14 + 8]
	cmp rax, r13
	je .LBB9_6
.LBB9_12:
	cmp r15d, 6
	ja .LBB9_15
	mov eax, 1
.LBB9_14:
	pause
	mov edx, eax
	mov ecx, r15d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB9_14
	jmp .LBB9_16
.LBB9_15:
	call rbx
	cmp r15d, 11
	jb .LBB9_16
	jmp .LBB9_17
.LBB9_18:
	mov edx, eax
	cmp r13, rdx
	sete cl
	mov rax, qword ptr [rsp + 32]
	not al
	test al, cl
	je .LBB9_19
	mov r14, qword ptr [rsp + 24]
	cmp qword ptr [r14], 0
	jne .LBB9_23
	lea rax, [r12 + 304]
	mov qword ptr [r14], rax
	mov rax, qword ptr [rsp + 8]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	mov byte ptr [r14 + 40], 2
.LBB9_23:
	mov rax, qword ptr [rsp + 16]
	mov qword ptr [rsp + 40], rax
	mov qword ptr [rsp + 48], rax
	mov qword ptr [rsp + 56], 0
	mov rdi, r14
	lea rsi, [rsp + 40]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov r15d, eax
	test al, al
	jne .LBB9_1
	cmp qword ptr [r14], 0
	jne .LBB9_25
.LBB9_26:
	mov qword ptr [r14], 0
	jmp .LBB9_1
.LBB9_25:
	mov rdi, r14
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_26
.LBB9_10:
	xor eax, eax
.LBB9_20:
	add rsp, 72
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_19:
	xor eax, eax
	cmp r13, rdx
	sete al
	inc rax
	jmp .LBB9_20
	mov qword ptr [r14], 0
	mov rdi, rax
	call _Unwind_Resume@PLT
