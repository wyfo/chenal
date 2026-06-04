chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov r12d, edx
	mov rax, qword ptr [rdi]
	mov rcx, qword ptr [rdi + 416]
	test rcx, rcx
	je .LBB1_2
	xor ecx, ecx
.LBB1_16:
	mov rax, qword ptr [rdi]
	mov esi, eax
	lea rdx, [4*rsi + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [rdi + 416], rdx
	mov rdx, rax
	mov eax, 1
	sete r8b
	shr rdx, 2
	test r8b, r8b
	cmovne rdx, rsi
	cmp r12, rdx
	je .LBB1_7
	shl rdx, 32
	or rdx, r12
	test cl, 1
	jne .LBB1_18
	jmp .LBB1_20
.LBB1_2:
	mov rbx, rsi
	lea r14, [rdi + 344]
	xor ecx, ecx
	mov r13, qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
.LBB1_3:
	cmp r12d, eax
	jne .LBB1_4
	test cl, 1
	jne .LBB1_6
	mov rcx, qword ptr [rdi + 376]
	cmp rcx, 2
	jne .LBB1_9
	mov rbp, qword ptr [rbx]
	cmp rbp, qword ptr [rdi + 352]
	mov r15, qword ptr [rbx + 8]
	sete al
	cmp qword ptr [rdi + 344], r15
	jne .LBB1_11
	test al, al
	je .LBB1_13
.LBB1_14:
	xor eax, eax
	xchg qword ptr [rdi + 376], rax
	mov cl, 1
	mov rax, qword ptr [rdi]
	mov rdx, qword ptr [rdi + 416]
	test rdx, rdx
	je .LBB1_3
	jmp .LBB1_16
.LBB1_11:
	xor eax, eax
	test al, al
	jne .LBB1_14
.LBB1_13:
	mov qword ptr [rsp], rdi
	mov rdi, r14
	call r13
	mov rdi, r15
	call qword ptr [rbp]
	mov rdi, qword ptr [rsp]
	mov qword ptr [rdi + 344], rdx
	mov qword ptr [rdi + 352], rax
	jmp .LBB1_14
.LBB1_9:
	mov rsi, qword ptr [rbx]
	mov rdx, qword ptr [rbx + 8]
	mov r15, rdi
	mov rdi, r14
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov rdi, r15
	mov ecx, eax
	mov rax, qword ptr [rdi]
	mov rdx, qword ptr [rdi + 416]
	test rdx, rdx
	je .LBB1_3
	jmp .LBB1_16
.LBB1_4:
	mov edx, eax
	shl rdx, 32
	or rdx, r12
	test cl, 1
	je .LBB1_20
.LBB1_18:
	mov rax, qword ptr [rdi + 376]
	cmp rax, 1
	ja .LBB1_20
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi + 376], rcx
.LBB1_20:
	xor eax, eax
.LBB1_7:
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB1_6:
	mov eax, 2
	jmp .LBB1_7
