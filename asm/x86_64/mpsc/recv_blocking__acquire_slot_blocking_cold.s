chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	push rax
	mov r13, r8
	mov r14, rcx
	mov r15, rdx
	mov r12, rsi
	mov qword ptr [rsp], rdi
	mov cl, 1
	xor ebp, ebp
	mov rbx, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	jmp .LBB6_1
.LBB6_25:
	mov rdi, r13
	call qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	xor ebp, ebp
	mov ecx, 0
	cmp al, 2
	jne .LBB6_26
.LBB6_1:
	test cl, 1
	jne .LBB6_3
	mov rax, qword ptr [r15 + 8]
	cmp rax, r14
	je .LBB6_11
.LBB6_3:
	mov rax, qword ptr [r12]
	mov rcx, qword ptr [r12 + 416]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r14, rdx
	jne .LBB6_4
	mov eax, eax
	cmp r14, rax
	jne .LBB6_16
	test bpl, 1
	jne .LBB6_25
	mov rsi, qword ptr [r12 + 384]
	cmp rsi, 2
	jne .LBB6_19
	mov rax, qword ptr [rbx]
	cmp rax, qword ptr [r12 + 360]
	mov rcx, qword ptr [r12 + 352]
	sete al
	cmp rcx, qword ptr [rbx + 8]
	jne .LBB6_21
	test al, al
	je .LBB6_23
.LBB6_24:
	xor ecx, ecx
	xor eax, eax
	xchg qword ptr [r12 + 384], rax
	mov bpl, 1
	jmp .LBB6_1
.LBB6_21:
	xor eax, eax
	test al, al
	jne .LBB6_24
.LBB6_23:
	lea rdi, [r12 + 352]
	call qword ptr [rip + spmc_waker::waker_cell::WakerCell::drop@GOTPCREL]
	mov rax, qword ptr [rbx]
	mov rdi, qword ptr [rbx + 8]
	call qword ptr [rax]
	mov qword ptr [r12 + 352], rdx
	mov qword ptr [r12 + 360], rax
	jmp .LBB6_24
.LBB6_19:
	lea rdi, [r12 + 352]
	call spmc_waker::SpmcWaker<_,_>::overwrite
	mov ebp, eax
	xor ecx, ecx
	jmp .LBB6_1
.LBB6_4:
	xor r13d, r13d
	mov rbx, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
	jmp .LBB6_5
.LBB6_9:
	inc r13d
.LBB6_10:
	mov rax, qword ptr [r15 + 8]
	cmp rax, r14
	je .LBB6_11
.LBB6_5:
	cmp r13d, 6
	ja .LBB6_8
	mov eax, 1
.LBB6_7:
	pause
	mov edx, eax
	mov ecx, r13d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB6_7
	jmp .LBB6_9
.LBB6_8:
	call rbx
	cmp r13d, 11
	jb .LBB6_9
	jmp .LBB6_10
.LBB6_11:
	test bpl, 1
	je .LBB6_12
	mov rax, qword ptr [r12 + 384]
	cmp rax, 1
	ja .LBB6_12
	mov rcx, rax
	or rcx, 2
	lock cmpxchg	qword ptr [r12 + 384], rcx
.LBB6_12:
	mov rax, qword ptr [rsp]
	mov qword ptr [rax], r15
	mov qword ptr [rax + 8], r14
	jmp .LBB6_27
.LBB6_16:
	mov rax, qword ptr [rsp]
	mov byte ptr [rax + 8], 0
	mov qword ptr [rax], 0
	jmp .LBB6_27
.LBB6_26:
	mov rcx, qword ptr [rsp]
	mov byte ptr [rcx + 8], al
	mov qword ptr [rcx], 0
.LBB6_27:
	add rsp, 8
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
