chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
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
	je .LBB1_3
.LBB1_1:
	lea rsi, [4*rdx + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 416], rsi
	sete sil
	shr rax, 2
	test sil, sil
	cmove rdx, rax
	cmp r13, rdx
	jne .LBB1_5
	mov byte ptr [rbx + 1], 0
.LBB1_21:
	mov al, 1
	jmp .LBB1_22
.LBB1_3:
	mov rbp, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	mov r12, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB1_4
.LBB1_16:
	xor eax, eax
	xchg qword ptr [r14 + 376], rax
	mov cl, 1
.LBB1_19:
	mov rax, qword ptr [r14 + 416]
	mov rdx, qword ptr [r14]
	mov edx, edx
	test rax, rax
	jne .LBB1_1
.LBB1_4:
	cmp r13, rdx
	jne .LBB1_5
	test cl, 1
	je .LBB1_10
	mov rdi, r15
	call rbp
	cmp al, 2
	jne .LBB1_20
	xor ecx, ecx
	jmp .LBB1_19
.LBB1_10:
	mov rsi, qword ptr [r14 + 376]
	cmp rsi, 2
	jne .LBB1_11
	mov rax, qword ptr [r12]
	cmp rax, qword ptr [r14 + 352]
	mov rcx, qword ptr [r14 + 344]
	sete al
	cmp rcx, qword ptr [r12 + 8]
	jne .LBB1_13
	test al, al
	jne .LBB1_16
.LBB1_15:
	lea rdi, [r14 + 344]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [r12]
	mov rdi, qword ptr [r12 + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 344], rdx
	mov qword ptr [r14 + 352], rax
	jmp .LBB1_16
.LBB1_13:
	xor eax, eax
	test al, al
	jne .LBB1_16
	jmp .LBB1_15
.LBB1_11:
	lea rdi, [r14 + 344]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov ecx, eax
	jmp .LBB1_19
.LBB1_5:
	shl rdx, 32
	or rdx, r13
	test cl, 1
	je .LBB1_6
	mov rax, qword ptr [r14 + 376]
	cmp rax, 1
	ja .LBB1_6
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [r14 + 376], rcx
.LBB1_6:
	mov qword ptr [rbx + 8], rdx
	xor eax, eax
.LBB1_22:
	mov byte ptr [rbx], al
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB1_20:
	mov byte ptr [rbx + 1], al
	jmp .LBB1_21
