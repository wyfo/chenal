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
	mov rax, qword ptr [rsi]
	mov rcx, qword ptr [rsi + 416]
	test rcx, rcx
	je .LBB1_4
	xor ecx, ecx
.LBB1_2:
	mov rax, qword ptr [r14]
	mov edx, eax
	lea rsi, [4*rdx + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 416], rsi
	sete sil
	shr rax, 2
	test sil, sil
	cmovne rax, rdx
	cmp r13, rax
	jne .LBB1_7
	mov byte ptr [rbx + 1], 0
.LBB1_23:
	mov al, 1
	jmp .LBB1_24
.LBB1_4:
	xor ecx, ecx
	mov rbp, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	mov r12, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB1_5
.LBB1_18:
	xor eax, eax
	xchg qword ptr [r14 + 376], rax
	mov cl, 1
.LBB1_21:
	mov rax, qword ptr [r14]
	mov rdx, qword ptr [r14 + 416]
	test rdx, rdx
	jne .LBB1_2
.LBB1_5:
	cmp r13d, eax
	jne .LBB1_6
	test cl, 1
	je .LBB1_12
	mov rdi, r15
	call rbp
	cmp al, 2
	jne .LBB1_22
	xor ecx, ecx
	jmp .LBB1_21
.LBB1_12:
	mov rsi, qword ptr [r14 + 376]
	cmp rsi, 2
	jne .LBB1_13
	mov rax, qword ptr [r12]
	cmp rax, qword ptr [r14 + 352]
	mov rcx, qword ptr [r14 + 344]
	sete al
	cmp rcx, qword ptr [r12 + 8]
	jne .LBB1_15
	test al, al
	jne .LBB1_18
.LBB1_17:
	lea rdi, [r14 + 344]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [r12]
	mov rdi, qword ptr [r12 + 8]
	call qword ptr [rax]
	mov qword ptr [r14 + 344], rdx
	mov qword ptr [r14 + 352], rax
	jmp .LBB1_18
.LBB1_15:
	xor eax, eax
	test al, al
	jne .LBB1_18
	jmp .LBB1_17
.LBB1_13:
	lea rdi, [r14 + 344]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov ecx, eax
	jmp .LBB1_21
.LBB1_6:
	mov eax, eax
.LBB1_7:
	shl rax, 32
	or rax, r13
	test cl, 1
	je .LBB1_8
	mov rcx, qword ptr [r14 + 376]
	cmp rcx, 1
	ja .LBB1_8
	mov rsi, rcx
	or rsi, 2
	mov rdx, rax
	mov rax, rcx
	lock cmpxchg	qword ptr [r14 + 376], rsi
	mov rax, rdx
.LBB1_8:
	mov qword ptr [rbx + 8], rax
	xor eax, eax
.LBB1_24:
	mov byte ptr [rbx], al
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB1_22:
	mov byte ptr [rbx + 1], al
	jmp .LBB1_23
