chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 56
	lea r15, [rsi + 256]
	lea r14, [rcx + 8]
	xor r9d, r9d
	movabs r12, -4294967296
	xorps xmm0, xmm0
	lea rbp, [rsp + 24]
.LBB10_1:
	mov rax, r8
	mov r11, qword ptr [rsi + 408]
	mov r8, qword ptr [rsi + 416]
	mov r10, r8
	and r10, rax
	dec r11
	cmp r10, r11
	seta r11b
	sbb r11b, 0
	je .LBB10_4
	movzx r8d, r11b
	cmp r8d, 255
	jne .LBB10_13
	lea r13, [rax + 1]
	jmp .LBB10_5
.LBB10_4:
	or r8d, eax
	inc r8d
	mov r13, rax
	and r13, r12
	or r13, r8
.LBB10_5:
	mov r11d, eax
	mov r8, rax
	shr r8, 32
	cmp r8d, eax
	jne .LBB10_6
	mov r8, qword ptr [rsi]
	cmp r8, rax
	jne .LBB10_1
	mov r8, qword ptr [rsi + 128]
	mov ebx, dword ptr [rsi + 416]
	add r8d, ebx
	inc r8d
	cmp r8, r11
	je .LBB10_14
	mov ebx, r13d
	shl r8, 32
	or r8, rbx
	mov r13, r8
.LBB10_6:
	lock cmpxchg	qword ptr [rsi], r13
	je .LBB10_11
	mov r8, rax
	jmp .LBB10_1
.LBB10_14:
	test r9b, 1
	jne .LBB10_18
	mov qword ptr [rsp + 16], rax
	mov qword ptr [rsp + 8], rsi
	mov qword ptr [rsp], rdi
	cmp qword ptr [rcx], 0
	jne .LBB10_17
	mov qword ptr [rcx], r15
	movups xmmword ptr [r14], xmm0
	mov qword ptr [r14 + 16], 0
	mov byte ptr [rcx + 40], 2
.LBB10_17:
	mov qword ptr [rsp + 24], rdx
	mov qword ptr [rsp + 32], rdx
	mov qword ptr [rsp + 40], 0
	mov rdi, rcx
	mov rsi, rbp
	mov r13, r15
	mov r15, rcx
	mov rbx, rdx
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	xorps xmm0, xmm0
	mov rdx, rbx
	mov rcx, r15
	mov r15, r13
	mov r9d, eax
	mov rdi, qword ptr [rsp]
	mov rsi, qword ptr [rsp + 8]
	mov r8, qword ptr [rsp + 16]
	jmp .LBB10_1
.LBB10_11:
	shl r10, 4
	add r10, qword ptr [rsi + 400]
	mov qword ptr [rdi + 8], r10
	mov qword ptr [rdi + 16], r11
	mov qword ptr [rdi], 0
	jmp .LBB10_12
.LBB10_13:
	xorps xmm0, xmm0
	movups xmmword ptr [rdi], xmm0
.LBB10_12:
	add rsp, 56
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB10_18:
	mov qword ptr [rdi], 1
	jmp .LBB10_12
