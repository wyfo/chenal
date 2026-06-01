chenal::channel::Chan<T,Ch,SP>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov r15, rcx
	mov r14, rsi
	mov rbx, rdi
	mov r13d, edx
	mov rax, qword ptr [rsi + 416]
	mov rcx, qword ptr [rsi]
	mov edx, ecx
	xor ecx, ecx
	test rax, rax
	je .LBB2_3
.LBB2_1:
	lea rsi, [4*rdx + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 416], rsi
	sete sil
	shr rax, 2
	test sil, sil
	cmove rdx, rax
	cmp r13, rdx
	jne .LBB2_5
	mov byte ptr [rbx + 1], 0
.LBB2_21:
	mov al, 1
	jmp .LBB2_22
.LBB2_3:
	mov rbp, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	mov r12, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB2_4
.LBB2_16:
	xor eax, eax
	xchg qword ptr [r14 + 376], rax
	mov cl, 1
.LBB2_19:
	mov rax, qword ptr [r14 + 416]
	mov rdx, qword ptr [r14]
	mov edx, edx
	test rax, rax
	jne .LBB2_1
.LBB2_4:
	cmp r13, rdx
	jne .LBB2_5
	test cl, 1
	je .LBB2_10
	mov rdi, r15
	call rbp
	cmp al, 2
	jne .LBB2_20
	xor ecx, ecx
	jmp .LBB2_19
.LBB2_10:
	mov rsi, qword ptr [r14 + 376]
	cmp rsi, 2
	jne .LBB2_11
	mov rax, qword ptr [r12]
	cmp rax, qword ptr [r14 + 352]
	mov rcx, qword ptr [r14 + 344]
	sete al
	cmp rcx, qword ptr [r12 + 8]
	jne .LBB2_13
	test al, al
	jne .LBB2_16
.LBB2_15:
	lea rdi, [r14 + 344]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [r12]
	mov rdi, qword ptr [r12 + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 344], rdx
	mov qword ptr [r14 + 352], rax
	jmp .LBB2_16
.LBB2_13:
	xor eax, eax
	test al, al
	jne .LBB2_16
	jmp .LBB2_15
.LBB2_11:
	lea rdi, [r14 + 344]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov ecx, eax
	jmp .LBB2_19
.LBB2_5:
	shl rdx, 32
	or rdx, r13
	test cl, 1
	je .LBB2_6
	mov rax, qword ptr [r14 + 376]
	cmp rax, 1
	ja .LBB2_6
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [r14 + 376], rcx
.LBB2_6:
	mov qword ptr [rbx + 8], rdx
	xor eax, eax
.LBB2_22:
	mov byte ptr [rbx], al
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB2_20:
	mov byte ptr [rbx + 1], al
	jmp .LBB2_21
