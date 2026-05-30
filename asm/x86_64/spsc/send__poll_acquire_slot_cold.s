chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold:
	mov rcx, qword ptr [rdi + 416]
	mov eax, 1
	test rcx, rcx
	je .LBB1_2
	ret
.LBB1_2:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov rbx, rsi
	mov r12d, edx
	lea r14, [rdi + 304]
	xor ecx, ecx
	mov r13, qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
.LBB1_3:
	mov rdx, qword ptr [rdi + 128]
	mov esi, dword ptr [rdi + 408]
	add edx, esi
	inc edx
	cmp rdx, r12
	jne .LBB1_12
	test cl, 1
	jne .LBB1_15
	mov rcx, qword ptr [rdi + 336]
	cmp rcx, 2
	jne .LBB1_11
	mov rbp, qword ptr [rbx]
	cmp rbp, qword ptr [rdi + 312]
	mov r15, qword ptr [rbx + 8]
	sete cl
	cmp qword ptr [rdi + 304], r15
	jne .LBB1_9
	test cl, cl
	je .LBB1_10
.LBB1_8:
	xor ecx, ecx
	xchg qword ptr [rdi + 336], rcx
	mov cl, 1
	mov rdx, qword ptr [rdi + 416]
	test rdx, rdx
	je .LBB1_3
	jmp .LBB1_16
.LBB1_9:
	xor ecx, ecx
	test cl, cl
	jne .LBB1_8
.LBB1_10:
	mov qword ptr [rsp], rdi
	mov rdi, r14
	call r13
	mov rdi, r15
	call qword ptr [rbp]
	mov rdi, qword ptr [rsp]
	mov rcx, rax
	mov eax, 1
	mov qword ptr [rdi + 304], rdx
	mov qword ptr [rdi + 312], rcx
	jmp .LBB1_8
.LBB1_11:
	mov rsi, qword ptr [rbx]
	mov rdx, qword ptr [rbx + 8]
	mov r15, rdi
	mov rdi, r14
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov rdi, r15
	mov ecx, eax
	mov eax, 1
	mov rdx, qword ptr [rdi + 416]
	test rdx, rdx
	je .LBB1_3
	jmp .LBB1_16
.LBB1_12:
	shl rdx, 32
	or rdx, r12
	test cl, 1
	je .LBB1_18
	mov rax, qword ptr [rdi + 336]
	cmp rax, 1
	ja .LBB1_18
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [rdi + 336], rcx
.LBB1_18:
	xor eax, eax
	jmp .LBB1_19
.LBB1_15:
	mov eax, 2
.LBB1_16:
.LBB1_19:
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
