chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 56
	lea rax, [rdi + 256]
	mov qword ptr [rsp + 16], rax
	lea r15, [rdx + 8]
	xor r11d, r11d
	movabs r12, -4294967296
	xorps xmm0, xmm0
	lea rbx, [rsp + 24]
.LBB9_1:
	mov rax, rcx
	mov r8, qword ptr [rdi + 408]
	mov rcx, qword ptr [rdi + 416]
	mov r13, rcx
	and r13, rax
	dec r8
	cmp r13, r8
	jne .LBB9_3
	or ecx, eax
	inc ecx
	mov rbp, rax
	and rbp, r12
	or rbp, rcx
	jmp .LBB9_4
.LBB9_3:
	lea rbp, [rax + 1]
.LBB9_4:
	mov r14d, eax
	mov rcx, rax
	shr rcx, 32
	cmp r14d, ecx
	jne .LBB9_5
	mov rcx, qword ptr [rdi + 128]
	cmp rcx, rax
	jne .LBB9_1
	mov rcx, qword ptr [rdi]
	mov r8, qword ptr [rdi + 424]
	test r8, r8
	je .LBB9_9
	mov rcx, qword ptr [rdi]
	mov r10d, ecx
	lea r8, [4*r10 + 2]
	mov rcx, rax
	mov eax, 1
	lock cmpxchg	qword ptr [rdi + 424], r8
	mov r8, rax
	mov rax, rcx
	mov r9d, 1
	sete cl
	shr r8, 2
	test cl, cl
	cmovne r8, r10
	cmp r14, r8
	jne .LBB9_12
	jmp .LBB9_16
.LBB9_9:
	cmp r14d, ecx
	je .LBB9_14
	mov r8d, ecx
.LBB9_12:
	mov ecx, ebp
	shl r8, 32
	or r8, rcx
	mov rbp, r8
.LBB9_5:
	mov rcx, qword ptr [rdi + 400]
	mov r10, qword ptr [rcx + 8*r13]
	#MEMBARRIER
	lock cmpxchg	qword ptr [rdi + 128], rbp
	je .LBB9_6
	mov rcx, rax
	jmp .LBB9_1
.LBB9_14:
	test r11b, 1
	jne .LBB9_15
	mov qword ptr [rsp + 8], rax
	mov r13, rdi
	cmp qword ptr [rdx], 0
	jne .LBB9_19
	mov rax, qword ptr [rsp + 16]
	mov qword ptr [rdx], rax
	movups xmmword ptr [r15], xmm0
	mov qword ptr [r15 + 16], 0
	mov byte ptr [rdx + 40], 2
.LBB9_19:
	mov qword ptr [rsp + 24], rsi
	mov qword ptr [rsp + 32], rsi
	mov qword ptr [rsp + 40], 0
	mov rdi, rdx
	mov rbp, rsi
	mov rsi, rbx
	mov r14, rdx
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	xorps xmm0, xmm0
	mov rsi, rbp
	mov rdx, r14
	mov r11d, eax
	mov rdi, r13
	mov rcx, qword ptr [rsp + 8]
	jmp .LBB9_1
.LBB9_6:
	xor r9d, r9d
.LBB9_16:
	mov rax, r9
	mov rdx, r10
	add rsp, 56
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_15:
	mov r9d, 2
	jmp .LBB9_16
