chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 40
	mov r12, rcx
	mov rbx, rsi
	mov r14, rdi
	lea rbp, [rdx + 8]
	xor r13d, r13d
	movabs r15, -4294967296
.LBB7_1:
	mov rax, r12
	jmp .LBB7_2
.LBB7_11:
	cmp r8d, r9d
	je .LBB7_12
.LBB7_15:
	mov eax, edi
	shl r9, 32
	or r9, rax
	mov rdi, r9
.LBB7_6:
	mov rax, qword ptr [r14 + 400]
	mov rcx, qword ptr [rax + 8*rsi]
	#MEMBARRIER
	mov rax, r12
	lock cmpxchg	qword ptr [r14 + 128], rdi
	je .LBB7_7
.LBB7_2:
	mov r12, rax
	mov rcx, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rsi, rax
	and rsi, r12
	dec rcx
	cmp rsi, rcx
	jne .LBB7_4
	or eax, r12d
	inc eax
	mov rdi, r12
	and rdi, r15
	or rdi, rax
	jmp .LBB7_5
.LBB7_4:
	lea rdi, [r12 + 1]
.LBB7_5:
	mov r8d, r12d
	mov rax, r12
	shr rax, 32
	cmp r8d, eax
	jne .LBB7_6
	mov rax, qword ptr [r14 + 128]
	cmp rax, r12
	jne .LBB7_2
	mov rax, qword ptr [r14 + 424]
	mov rcx, qword ptr [r14]
	mov r9d, ecx
	test rax, rax
	je .LBB7_11
	lea rcx, [4*r9 + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 424], rcx
	mov rcx, rax
	mov eax, 1
	sete r10b
	shr rcx, 2
	test r10b, r10b
	cmove r9, rcx
	cmp r8, r9
	jne .LBB7_15
	jmp .LBB7_8
.LBB7_12:
	test r13b, 1
	jne .LBB7_13
	cmp qword ptr [rdx], 0
	jne .LBB7_18
	lea rax, [r14 + 256]
	mov qword ptr [rdx], rax
	xorps xmm0, xmm0
	movups xmmword ptr [rbp], xmm0
	mov qword ptr [rbp + 16], 0
	mov byte ptr [rdx + 40], 2
.LBB7_18:
	mov qword ptr [rsp + 8], rbx
	mov qword ptr [rsp + 16], rbx
	mov qword ptr [rsp + 24], 0
	mov rdi, rdx
	lea rsi, [rsp + 8]
	mov qword ptr [rsp], rdx
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov rdx, qword ptr [rsp]
	mov r13d, eax
	test al, al
	jne .LBB7_1
	cmp qword ptr [rdx], 0
	jne .LBB7_20
.LBB7_21:
	mov rdx, qword ptr [rsp]
	mov qword ptr [rdx], 0
	jmp .LBB7_1
.LBB7_20:
	mov rdi, qword ptr [rsp]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB7_21
.LBB7_7:
	xor eax, eax
.LBB7_8:
	mov rdx, rcx
	add rsp, 40
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB7_13:
	mov eax, 2
	jmp .LBB7_8
	mov rcx, qword ptr [rsp]
	mov qword ptr [rcx], 0
	mov rdi, rax
	call _Unwind_Resume@PLT
