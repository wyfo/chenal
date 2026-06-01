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
	lea r12, [rdx + 8]
	xor r9d, r9d
	movabs r13, -4294967296
	xorps xmm0, xmm0
	lea rbx, [rsp + 24]
.LBB9_1:
	mov rax, rcx
	mov r8, qword ptr [rdi + 408]
	mov rcx, qword ptr [rdi + 416]
	mov r10, rcx
	and r10, rax
	dec r8
	cmp r10, r8
	jne .LBB9_3
	or ecx, eax
	inc ecx
	mov r11, rax
	and r11, r13
	or r11, rcx
	jmp .LBB9_4
.LBB9_3:
	lea r11, [rax + 1]
.LBB9_4:
	mov ebp, eax
	mov rcx, rax
	shr rcx, 32
	cmp ebp, ecx
	jne .LBB9_5
	mov rcx, qword ptr [rdi + 128]
	cmp rcx, rax
	jne .LBB9_1
	mov rcx, qword ptr [rdi + 424]
	mov r8, qword ptr [rdi]
	mov r15d, r8d
	test rcx, rcx
	je .LBB9_9
	lea rcx, [4*r15 + 2]
	mov r8, rax
	mov eax, 1
	lock cmpxchg	qword ptr [rdi + 424], rcx
	mov rcx, rax
	mov rax, r8
	mov r8d, 1
	sete r14b
	shr rcx, 2
	test r14b, r14b
	cmove r15, rcx
	cmp rbp, r15
	jne .LBB9_13
	jmp .LBB9_17
.LBB9_9:
	cmp ebp, r15d
	je .LBB9_10
.LBB9_13:
	#MEMBARRIER
	mov ecx, r11d
	shl r15, 32
	or r15, rcx
	mov r11, r15
.LBB9_5:
	mov rcx, qword ptr [rdi + 400]
	#MEMBARRIER
	mov r14, qword ptr [rcx + 8*r10]
	#MEMBARRIER
	lock cmpxchg	qword ptr [rdi + 128], r11
	je .LBB9_14
	mov rcx, rax
	jmp .LBB9_1
.LBB9_10:
	test r9b, 1
	jne .LBB9_11
	mov qword ptr [rsp + 8], rax
	mov r15, rdi
	cmp qword ptr [rdx], 0
	jne .LBB9_20
	mov rax, qword ptr [rsp + 16]
	mov qword ptr [rdx], rax
	movups xmmword ptr [r12], xmm0
	mov qword ptr [r12 + 16], 0
	mov byte ptr [rdx + 40], 2
.LBB9_20:
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
	mov r9d, eax
	mov rdi, r15
	mov rcx, qword ptr [rsp + 8]
	jmp .LBB9_1
.LBB9_14:
	mov rsi, qword ptr [rdi + 384]
	cmp rsi, 1
	jbe .LBB9_15
.LBB9_16:
	xor r8d, r8d
.LBB9_17:
	mov rax, r8
	mov rdx, r14
	add rsp, 56
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_15:
	add rdi, 352
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	jmp .LBB9_16
.LBB9_11:
	mov r8d, 2
	jmp .LBB9_17
