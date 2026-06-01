chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 72
	mov r12, rcx
	mov qword ptr [rsp + 24], rsi
	mov rbx, rdi
	mov qword ptr [rsp + 32], rdx
	lea rax, [rdx + 8]
	mov qword ptr [rsp + 16], rax
	mov dword ptr [rsp + 12], 0
	mov r15, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
.LBB11_1:
	mov rax, r12
	jmp .LBB11_3
.LBB11_2:
	lea rcx, [r12 + 1]
	mov rax, r12
	lock cmpxchg	qword ptr [rbx + 128], rcx
	je .LBB11_20
.LBB11_3:
	mov r12, rax
	mov rbp, qword ptr [rbx + 432]
	and rbp, rax
	mov rax, qword ptr [rbx + 416]
	mov rcx, rbp
	shl rcx, 4
	lea r14, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r12
	jne .LBB11_6
.LBB11_4:
	mov r13, qword ptr [r14]
	#MEMBARRIER
	mov rax, qword ptr [rbx + 424]
	dec rax
	cmp rbp, rax
	jne .LBB11_2
	mov rcx, qword ptr [rbx + 432]
	or rcx, r12
	inc rcx
	mov rax, r12
	lock cmpxchg	qword ptr [rbx + 128], rcx
	jne .LBB11_3
	jmp .LBB11_20
.LBB11_6:
	mov rax, qword ptr [rbx + 128]
	cmp rax, r12
	jne .LBB11_3
	mov rax, qword ptr [rbx]
	mov rcx, qword ptr [rbx + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r12, rdx
	je .LBB11_16
	xor r13d, r13d
.LBB11_9:
	cmp r13d, 6
	ja .LBB11_12
	mov eax, 1
.LBB11_11:
	pause
	mov edx, eax
	mov ecx, r13d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB11_11
	jmp .LBB11_13
.LBB11_12:
	call r15
	cmp r13d, 11
	jae .LBB11_14
.LBB11_13:
	inc r13d
.LBB11_14:
	mov rax, qword ptr [rbx + 128]
	cmp rax, r12
	jne .LBB11_3
	mov rax, qword ptr [r14 + 8]
	cmp rax, r12
	jne .LBB11_9
	jmp .LBB11_4
.LBB11_16:
	mov ecx, eax
	cmp r12, rcx
	sete al
	mov edx, dword ptr [rsp + 12]
	not dl
	test dl, al
	je .LBB11_23
	mov rdi, qword ptr [rsp + 32]
	cmp qword ptr [rdi], 0
	jne .LBB11_19
	lea rax, [rbx + 304]
	mov qword ptr [rdi], rax
	mov rax, qword ptr [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	mov byte ptr [rdi + 40], 2
.LBB11_19:
	mov rax, qword ptr [rsp + 24]
	mov qword ptr [rsp + 40], rax
	mov qword ptr [rsp + 48], rax
	mov qword ptr [rsp + 56], 0
	lea rsi, [rsp + 40]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov dword ptr [rsp + 12], eax
	jmp .LBB11_1
.LBB11_20:
	mov rax, qword ptr [rbx + 256]
	test al, 1
	jne .LBB11_24
.LBB11_21:
	xor eax, eax
.LBB11_22:
	mov rdx, r13
	add rsp, 72
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB11_23:
	xor eax, eax
	cmp r12, rcx
	sete al
	inc rax
	jmp .LBB11_22
.LBB11_24:
	add rbx, 256
	mov rdi, rbx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	jmp .LBB11_21
