chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 104
	mov qword ptr [rsp + 8], rcx
	mov r12, rdx
	mov r15, rsi
	mov qword ptr [rsp], rdi
	mov qword ptr [rsp + 24], 0
	xor ecx, ecx
	mov r14, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
.LBB9_1:
	mov qword ptr [rsp + 16], rcx
	mov rax, r12
	jmp .LBB9_2
.LBB9_12:
	lea rdx, [r12 + 1]
	mov rax, r12
	lock cmpxchg	qword ptr [r15 + 128], rdx
	je .LBB9_14
.LBB9_2:
	mov r12, rax
	mov rbx, qword ptr [r15 + 432]
	and rbx, rax
	mov rax, qword ptr [r15 + 416]
	mov rcx, rbx
	shl rcx, 4
	lea r13, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r12
	jne .LBB9_3
.LBB9_10:
	mov rcx, qword ptr [r13]
	#MEMBARRIER
	mov rax, qword ptr [r15 + 424]
	dec rax
	cmp rbx, rax
	jne .LBB9_12
	mov rdx, qword ptr [r15 + 432]
	or rdx, r12
	inc rdx
	mov rax, r12
	lock cmpxchg	qword ptr [r15 + 128], rdx
	jne .LBB9_2
	jmp .LBB9_14
.LBB9_3:
	mov rax, qword ptr [r15 + 128]
	cmp rax, r12
	jne .LBB9_2
	mov rax, qword ptr [r15]
	mov rcx, qword ptr [r15 + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r12, rdx
	je .LBB9_23
	xor ebp, ebp
.LBB9_6:
	cmp rbp, 5
	jbe .LBB9_7
	call r14
	mov rax, qword ptr [r15 + 128]
	cmp rax, r12
	je .LBB9_17
	jmp .LBB9_2
.LBB9_7:
	mov eax, 1
.LBB9_8:
	pause
	mov edx, eax
	mov ecx, ebp
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB9_8
	inc rbp
	mov rax, qword ptr [r15 + 128]
	cmp rax, r12
	jne .LBB9_2
.LBB9_17:
	mov rax, qword ptr [r13 + 8]
	cmp rax, r12
	jne .LBB9_6
	jmp .LBB9_10
.LBB9_23:
	mov eax, eax
	cmp r12, rax
	jne .LBB9_24
	test byte ptr [rsp + 16], 1
	je .LBB9_30
	mov rdi, qword ptr [rsp + 8]
	call qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	xor ecx, ecx
	cmp al, 2
	je .LBB9_1
	jmp .LBB9_36
.LBB9_30:
	cmp qword ptr [rsp + 24], 0
	jne .LBB9_32
	lea rax, [rsp + 32]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r15 + 304]
	mov qword ptr [rsp + 24], rax
	mov byte ptr [rsp + 64], 2
.LBB9_32:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 72], rax
	mov qword ptr [rsp + 80], rax
	mov qword ptr [rsp + 88], 0
	lea rdi, [rsp + 24]
	lea rsi, [rsp + 72]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov ecx, eax
	jmp .LBB9_1
.LBB9_14:
	mov rdx, qword ptr [rsp]
	mov qword ptr [rdx + 8], rcx
	xor eax, eax
	mov byte ptr [rdx], al
	cmp qword ptr [rsp + 24], 0
	jne .LBB9_27
.LBB9_28:
	add rsp, 104
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_24:
	mov rdx, qword ptr [rsp]
	mov byte ptr [rdx + 1], 0
	jmp .LBB9_25
.LBB9_36:
	mov rdx, qword ptr [rsp]
	mov byte ptr [rdx + 1], al
.LBB9_25:
	mov al, 1
	mov byte ptr [rdx], al
	cmp qword ptr [rsp + 24], 0
	je .LBB9_28
.LBB9_27:
	lea rdi, [rsp + 24]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_28
	jmp .LBB9_21
.LBB9_21:
	mov rbx, rax
	cmp qword ptr [rsp + 24], 0
	jne .LBB9_18
.LBB9_22:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB9_18:
	lea rdi, [rsp + 24]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_22
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
