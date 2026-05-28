chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 104
	mov qword ptr [rsp + 64], rcx
	mov r12, rdx
	mov r15, rsi
	mov qword ptr [rsp], rdi
	mov qword ptr [rsp + 16], 0
	xor ecx, ecx
	mov rbp, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
.LBB8_1:
	mov dword ptr [rsp + 12], ecx
	mov rax, r12
	jmp .LBB8_2
.LBB8_8:
	lea rdx, [r12 + 1]
	mov rax, r12
	lock cmpxchg	qword ptr [r15 + 128], rdx
	je .LBB8_10
.LBB8_2:
	mov r12, rax
	mov r14, qword ptr [r15 + 432]
	and r14, rax
	mov rax, qword ptr [r15 + 416]
	mov rcx, r14
	shl rcx, 4
	lea rbx, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r12
	jne .LBB8_3
.LBB8_6:
	mov rcx, qword ptr [rbx]
	#MEMBARRIER
	mov rax, qword ptr [r15 + 424]
	dec rax
	cmp r14, rax
	jne .LBB8_8
	mov rdx, qword ptr [r15 + 432]
	or rdx, r12
	inc rdx
	mov rax, r12
	lock cmpxchg	qword ptr [r15 + 128], rdx
	jne .LBB8_2
	jmp .LBB8_10
.LBB8_3:
	mov rax, qword ptr [r15 + 128]
	cmp rax, r12
	jne .LBB8_2
	mov rax, qword ptr [r15]
	mov rcx, qword ptr [r15 + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r12, rdx
	je .LBB8_24
	mov rax, qword ptr [rbx + 8]
	cmp rax, r12
	je .LBB8_6
	xor r13d, r13d
	jmp .LBB8_12
.LBB8_17:
	inc r13d
.LBB8_18:
	mov rax, qword ptr [rbx + 8]
	cmp rax, r12
	je .LBB8_6
.LBB8_12:
	cmp r13d, 6
	ja .LBB8_15
	mov eax, 1
.LBB8_14:
	pause
	mov edx, eax
	mov ecx, r13d
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB8_14
	jmp .LBB8_17
.LBB8_15:
	call rbp
	cmp r13d, 11
	jb .LBB8_17
	jmp .LBB8_18
.LBB8_24:
	mov eax, eax
	cmp r12, rax
	jne .LBB8_25
	test byte ptr [rsp + 12], 1
	je .LBB8_31
	mov rdi, qword ptr [rsp + 64]
	call qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	xor ecx, ecx
	cmp al, 2
	je .LBB8_1
	jmp .LBB8_40
.LBB8_31:
	cmp qword ptr [rsp + 16], 0
	jne .LBB8_33
	lea rax, [rsp + 24]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r15 + 304]
	mov qword ptr [rsp + 16], rax
	mov byte ptr [rsp + 56], 2
.LBB8_33:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 72], rax
	mov qword ptr [rsp + 80], rax
	mov qword ptr [rsp + 88], 0
	lea rdi, [rsp + 16]
	lea rsi, [rsp + 72]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov cl, 1
	test al, al
	jne .LBB8_1
	cmp qword ptr [rsp + 16], 0
	jne .LBB8_36
.LBB8_37:
	mov qword ptr [rsp + 16], 0
	xor ecx, ecx
	jmp .LBB8_1
.LBB8_36:
	lea rdi, [rsp + 16]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB8_37
.LBB8_10:
	mov rdx, qword ptr [rsp]
	mov qword ptr [rdx + 8], rcx
	xor eax, eax
	mov byte ptr [rdx], al
	cmp qword ptr [rsp + 16], 0
	jne .LBB8_28
.LBB8_29:
	add rsp, 104
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB8_25:
	mov rdx, qword ptr [rsp]
	mov byte ptr [rdx + 1], 0
	jmp .LBB8_26
.LBB8_40:
	mov rdx, qword ptr [rsp]
	mov byte ptr [rdx + 1], al
.LBB8_26:
	mov al, 1
	mov byte ptr [rdx], al
	cmp qword ptr [rsp + 16], 0
	je .LBB8_29
.LBB8_28:
	lea rdi, [rsp + 16]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB8_29
	jmp .LBB8_20
.LBB8_20:
	mov rbx, rax
	cmp qword ptr [rsp + 16], 0
	jne .LBB8_21
.LBB8_22:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB8_21:
	lea rdi, [rsp + 16]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB8_22
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
