chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 24
	mov r14, r8
	mov r15, rcx
	mov r12, rsi
	mov qword ptr [rsp + 8], rdi
	lea rdi, [rsi + 352]
	mov al, 1
	xor ebp, ebp
	mov r13, qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
.LBB6_1:
	test al, 1
	jne .LBB6_3
	mov rax, qword ptr [r15 + 8]
	cmp rax, r14
	je .LBB6_12
.LBB6_3:
	mov rax, qword ptr [r12]
	mov rcx, qword ptr [r12 + 416]
	shr rcx
	mov esi, -2
	sub esi, ecx
	and esi, eax
	cmp r14, rsi
	jne .LBB6_4
	mov eax, eax
	cmp r14, rax
	jne .LBB6_18
	test bpl, 1
	jne .LBB6_27
	mov rcx, qword ptr [r12 + 384]
	cmp rcx, 2
	jne .LBB6_21
	mov rbx, qword ptr [rdx]
	cmp rbx, qword ptr [r12 + 360]
	mov rbp, qword ptr [rdx + 8]
	sete al
	cmp qword ptr [r12 + 352], rbp
	jne .LBB6_23
	test al, al
	je .LBB6_25
.LBB6_26:
	xor eax, eax
	xor ecx, ecx
	xchg qword ptr [r12 + 384], rcx
	mov bpl, 1
	jmp .LBB6_1
.LBB6_23:
	xor eax, eax
	test al, al
	jne .LBB6_26
.LBB6_25:
	lea rdi, [r12 + 352]
	mov qword ptr [rsp + 16], rdx
	call r13
	mov rdi, rbp
	call qword ptr [rbx]
	lea rdi, [r12 + 352]
	mov rcx, rdx
	mov rdx, qword ptr [rsp + 16]
	mov qword ptr [r12 + 352], rcx
	mov qword ptr [r12 + 360], rax
	jmp .LBB6_26
.LBB6_21:
	mov rsi, qword ptr [rdx]
	mov rax, qword ptr [rdx + 8]
	mov rbp, rdi
	mov rbx, rdx
	mov rdx, rax
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov rdi, rbp
	mov rdx, rbx
	mov ebp, eax
	xor eax, eax
	jmp .LBB6_1
.LBB6_4:
	mov rax, qword ptr [r15 + 8]
	cmp rax, r14
	jne .LBB6_5
.LBB6_12:
	test bpl, 1
	je .LBB6_13
	mov rax, qword ptr [r12 + 384]
	cmp rax, 1
	ja .LBB6_13
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [r12 + 384], rcx
.LBB6_13:
	mov rax, qword ptr [rsp + 8]
	mov qword ptr [rax + 8], r15
	mov qword ptr [rax + 16], r14
	mov qword ptr [rax], 0
	jmp .LBB6_14
.LBB6_18:
	xorps xmm0, xmm0
	mov rax, qword ptr [rsp + 8]
	movups xmmword ptr [rax], xmm0
	jmp .LBB6_14
.LBB6_27:
	mov rax, qword ptr [rsp + 8]
	mov qword ptr [rax], 1
.LBB6_14:
	add rsp, 24
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB6_5:
	xor r13d, r13d
	mov rbx, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
	jmp .LBB6_6
.LBB6_10:
	inc r13d
.LBB6_11:
	mov rax, qword ptr [r15 + 8]
	cmp rax, r14
	je .LBB6_12
.LBB6_6:
	cmp r13d, 6
	ja .LBB6_9
	mov eax, 1
.LBB6_8:
	pause
	mov edx, eax
	mov ecx, r13d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB6_8
	jmp .LBB6_10
.LBB6_9:
	call rbx
	cmp r13d, 11
	jb .LBB6_10
	jmp .LBB6_11
