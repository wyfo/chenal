chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold:
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
	xor r10d, r10d
	movabs r12, -4294967296
	xorps xmm0, xmm0
	lea rbx, [rsp + 24]
.LBB10_1:
	mov rax, rcx
	mov r8, qword ptr [rdi + 408]
	mov rcx, qword ptr [rdi + 416]
	mov r11, rcx
	and r11, rax
	dec r8
	cmp r11, r8
	jne .LBB10_3
	or ecx, eax
	inc ecx
	mov r13, rax
	and r13, r12
	or r13, rcx
	jmp .LBB10_4
.LBB10_3:
	lea r13, [rax + 1]
.LBB10_4:
	mov ebp, eax
	mov rcx, rax
	shr rcx, 32
	cmp ebp, ecx
	jne .LBB10_5
	mov rcx, qword ptr [rdi + 128]
	cmp rcx, rax
	jne .LBB10_1
	mov rcx, qword ptr [rdi + 424]
	mov r8, qword ptr [rdi]
	mov r14d, r8d
	test rcx, rcx
	je .LBB10_10
	lea rcx, [4*r14 + 2]
	mov r8, rax
	mov eax, 1
	lock cmpxchg	qword ptr [rdi + 424], rcx
	mov rcx, rax
	mov rax, r8
	mov r8d, 1
	sete r9b
	shr rcx, 2
	test r9b, r9b
	cmove r14, rcx
	cmp rbp, r14
	jne .LBB10_14
	jmp .LBB10_7
.LBB10_10:
	cmp ebp, r14d
	je .LBB10_11
.LBB10_14:
	mov ecx, r13d
	shl r14, 32
	or r14, rcx
	mov r13, r14
.LBB10_5:
	mov rcx, qword ptr [rdi + 400]
	mov r9, qword ptr [rcx + 8*r11]
	#MEMBARRIER
	lock cmpxchg	qword ptr [rdi + 128], r13
	je .LBB10_6
	mov rcx, rax
	jmp .LBB10_1
.LBB10_11:
	test r10b, 1
	jne .LBB10_12
	mov qword ptr [rsp + 8], rax
	mov r13, rdi
	cmp qword ptr [rdx], 0
	jne .LBB10_18
	mov rax, qword ptr [rsp + 16]
	mov qword ptr [rdx], rax
	movups xmmword ptr [r15], xmm0
	mov qword ptr [r15 + 16], 0
	mov byte ptr [rdx + 40], 2
.LBB10_18:
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
	mov r10d, eax
	mov rdi, r13
	mov rcx, qword ptr [rsp + 8]
	jmp .LBB10_1
.LBB10_6:
	xor r8d, r8d
.LBB10_7:
	mov rax, r8
	mov rdx, r9
	add rsp, 56
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB10_12:
	mov r8d, 2
	jmp .LBB10_7
