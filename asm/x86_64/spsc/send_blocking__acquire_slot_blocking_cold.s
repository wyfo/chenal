chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov rbx, rdi
	mov rax, qword ptr [rsi + 416]
	test rax, rax
	je .LBB2_1
.LBB2_18:
	mov byte ptr [rbx + 1], 0
.LBB2_20:
	mov al, 1
.LBB2_21:
	mov byte ptr [rbx], al
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB2_1:
	mov r15, rcx
	mov r14, rsi
	mov r13d, edx
	xor eax, eax
	mov rbp, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	mov r12, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB2_2
.LBB2_14:
	xor eax, eax
	xchg qword ptr [r14 + 336], rax
	mov al, 1
.LBB2_17:
	mov rcx, qword ptr [r14 + 416]
	test rcx, rcx
	jne .LBB2_18
.LBB2_2:
	mov rcx, qword ptr [r14 + 128]
	mov edx, dword ptr [r14 + 408]
	add ecx, edx
	inc ecx
	cmp rcx, r13
	jne .LBB2_3
	test al, 1
	je .LBB2_8
	mov rdi, r15
	call rbp
	cmp al, 2
	jne .LBB2_19
	xor eax, eax
	jmp .LBB2_17
.LBB2_8:
	mov rsi, qword ptr [r14 + 336]
	cmp rsi, 2
	jne .LBB2_9
	mov rax, qword ptr [r12]
	cmp rax, qword ptr [r14 + 312]
	mov rcx, qword ptr [r14 + 304]
	sete al
	cmp rcx, qword ptr [r12 + 8]
	jne .LBB2_11
	test al, al
	jne .LBB2_14
.LBB2_13:
	lea rdi, [r14 + 304]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [r12]
	mov rdi, qword ptr [r12 + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 304], rdx
	mov qword ptr [r14 + 312], rax
	jmp .LBB2_14
.LBB2_11:
	xor eax, eax
	test al, al
	jne .LBB2_14
	jmp .LBB2_13
.LBB2_9:
	lea rdi, [r14 + 304]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	jmp .LBB2_17
.LBB2_3:
	shl rcx, 32
	or rcx, r13
	test al, 1
	je .LBB2_4
	mov rax, qword ptr [r14 + 336]
	cmp rax, 1
	ja .LBB2_4
	mov rdx, rax
	or rdx, 2
	lock cmpxchg	qword ptr [r14 + 336], rdx
.LBB2_4:
	mov qword ptr [rbx + 8], rcx
	xor eax, eax
	jmp .LBB2_21
.LBB2_19:
	mov byte ptr [rbx + 1], al
	jmp .LBB2_20
