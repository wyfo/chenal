chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov rbx, rdi
	mov rax, qword ptr [rsi + 424]
	test rax, rax
	je .LBB4_1
.LBB4_18:
	mov byte ptr [rbx + 1], 0
.LBB4_20:
	mov al, 1
.LBB4_21:
	mov byte ptr [rbx], al
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB4_1:
	mov r15, rcx
	mov r14, rsi
	mov r13d, edx
	xor eax, eax
	mov rbp, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	mov r12, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB4_2
.LBB4_14:
	xor eax, eax
	xchg qword ptr [r14 + 384], rax
	mov al, 1
.LBB4_17:
	mov rcx, qword ptr [r14 + 424]
	test rcx, rcx
	jne .LBB4_18
.LBB4_2:
	mov rcx, qword ptr [r14 + 128]
	mov edx, dword ptr [r14 + 416]
	add ecx, edx
	inc ecx
	cmp rcx, r13
	jne .LBB4_3
	test al, 1
	je .LBB4_8
	mov rdi, r15
	call rbp
	cmp al, 2
	jne .LBB4_19
	xor eax, eax
	jmp .LBB4_17
.LBB4_8:
	mov rsi, qword ptr [r14 + 384]
	cmp rsi, 2
	jne .LBB4_9
	mov rax, qword ptr [r12]
	cmp rax, qword ptr [r14 + 360]
	mov rcx, qword ptr [r14 + 352]
	sete al
	cmp rcx, qword ptr [r12 + 8]
	jne .LBB4_11
	test al, al
	jne .LBB4_14
.LBB4_13:
	lea rdi, [r14 + 352]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [r12]
	mov rdi, qword ptr [r12 + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 352], rdx
	mov qword ptr [r14 + 360], rax
	jmp .LBB4_14
.LBB4_11:
	xor eax, eax
	test al, al
	jne .LBB4_14
	jmp .LBB4_13
.LBB4_9:
	lea rdi, [r14 + 352]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	jmp .LBB4_17
.LBB4_3:
	shl rcx, 32
	or rcx, r13
	test al, 1
	je .LBB4_4
	mov rax, qword ptr [r14 + 384]
	cmp rax, 1
	ja .LBB4_4
	mov rdx, rax
	or rdx, 2
	lock cmpxchg	qword ptr [r14 + 384], rdx
.LBB4_4:
	mov qword ptr [rbx + 8], rcx
	xor eax, eax
	jmp .LBB4_21
.LBB4_19:
	mov byte ptr [rbx + 1], al
	jmp .LBB4_20
