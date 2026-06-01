chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov r12d, edx
	mov rax, qword ptr [rdi + 416]
	mov rcx, qword ptr [rdi]
	mov edx, ecx
	test rax, rax
	je .LBB1_2
	xor ecx, ecx
.LBB1_15:
	lea rsi, [4*rdx + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [rdi + 416], rsi
	mov rsi, rax
	mov eax, 1
	sete r8b
	shr rsi, 2
	test r8b, r8b
	cmove rdx, rsi
	cmp r12, rdx
	je .LBB1_6
.LBB1_16:
	shl rdx, 32
	or rdx, r12
	test cl, 1
	je .LBB1_19
	mov rax, qword ptr [rdi + 376]
	cmp rax, 1
	ja .LBB1_19
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi + 376], rcx
.LBB1_19:
	xor eax, eax
.LBB1_6:
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB1_2:
	mov rbx, rsi
	lea r14, [rdi + 344]
	xor ecx, ecx
	mov r13, qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
.LBB1_3:
	cmp r12, rdx
	jne .LBB1_16
	test cl, 1
	jne .LBB1_5
	mov rcx, qword ptr [rdi + 376]
	cmp rcx, 2
	jne .LBB1_8
	mov rbp, qword ptr [rbx]
	cmp rbp, qword ptr [rdi + 352]
	mov r15, qword ptr [rbx + 8]
	sete al
	cmp qword ptr [rdi + 344], r15
	jne .LBB1_10
	test al, al
	je .LBB1_12
.LBB1_13:
	xor eax, eax
	xchg qword ptr [rdi + 376], rax
	mov cl, 1
.LBB1_14:
	mov rax, qword ptr [rdi + 416]
	mov rdx, qword ptr [rdi]
	mov edx, edx
	test rax, rax
	je .LBB1_3
	jmp .LBB1_15
.LBB1_10:
	xor eax, eax
	test al, al
	jne .LBB1_13
.LBB1_12:
	mov qword ptr [rsp], rdi
	mov rdi, r14
	call r13
	mov rdi, r15
	call qword ptr [rbp]
	mov rdi, qword ptr [rsp]
	mov qword ptr [rdi + 344], rdx
	mov qword ptr [rdi + 352], rax
	jmp .LBB1_13
.LBB1_8:
	mov rsi, qword ptr [rbx]
	mov rdx, qword ptr [rbx + 8]
	mov r15, rdi
	mov rdi, r14
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov rdi, r15
	mov ecx, eax
	jmp .LBB1_14
.LBB1_5:
	mov eax, 2
	jmp .LBB1_6
