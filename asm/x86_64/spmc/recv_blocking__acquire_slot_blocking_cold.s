chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 88
	mov r15, rdx
	mov r14, rsi
	mov rbx, rdi
	mov qword ptr [rsp + 8], 0
	xor edx, edx
	movabs rbp, -4294967296
.LBB9_1:
	mov r11, qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
.LBB9_2:
	mov rax, r15
	jmp .LBB9_3
.LBB9_13:
	cmp r8d, r9d
	je .LBB9_14
.LBB9_19:
	#MEMBARRIER
	mov eax, edi
	shl r9, 32
	or r9, rax
	mov rdi, r9
.LBB9_7:
	mov rax, qword ptr [r14 + 400]
	#MEMBARRIER
	mov r13, qword ptr [rax + 8*rsi]
	#MEMBARRIER
	mov rax, r15
	lock cmpxchg	qword ptr [r14 + 128], rdi
	je .LBB9_8
.LBB9_3:
	mov r15, rax
	mov rdi, qword ptr [r14 + 408]
	mov rax, qword ptr [r14 + 416]
	mov rsi, rax
	and rsi, r15
	dec rdi
	cmp rsi, rdi
	jne .LBB9_5
	or eax, r15d
	inc eax
	mov rdi, r15
	and rdi, rbp
	or rdi, rax
	jmp .LBB9_6
.LBB9_5:
	lea rdi, [r15 + 1]
.LBB9_6:
	mov r8d, r15d
	mov rax, r15
	shr rax, 32
	cmp r8d, eax
	jne .LBB9_7
	mov rax, qword ptr [r14 + 128]
	cmp rax, r15
	jne .LBB9_3
	mov rax, qword ptr [r14 + 424]
	mov r9, qword ptr [r14]
	mov r9d, r9d
	test rax, rax
	je .LBB9_13
	lea r10, [4*r9 + 2]
	mov eax, 1
	lock cmpxchg	qword ptr [r14 + 424], r10
	sete r10b
	shr rax, 2
	test r10b, r10b
	cmove r9, rax
	cmp r8, r9
	jne .LBB9_19
	jmp .LBB9_25
.LBB9_14:
	test dl, 1
	je .LBB9_30
	mov r13, rcx
	mov rdi, rcx
	mov r12, r11
	call r11
	xor edx, edx
	cmp al, 2
	mov rcx, r13
	mov r11, r12
	je .LBB9_2
	jmp .LBB9_17
.LBB9_30:
	mov r13, rcx
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_32
	lea rax, [rsp + 16]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r14 + 256]
	mov qword ptr [rsp + 8], rax
	mov byte ptr [rsp + 48], 2
.LBB9_32:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 56], rax
	mov qword ptr [rsp + 64], rax
	mov qword ptr [rsp + 72], 0
	lea rdi, [rsp + 8]
	lea rsi, [rsp + 56]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov edx, eax
	mov rcx, r13
	jmp .LBB9_1
.LBB9_8:
	mov rsi, qword ptr [r14 + 384]
	cmp rsi, 1
	ja .LBB9_10
	add r14, 352
	mov rdi, r14
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
.LBB9_10:
	mov qword ptr [rbx + 8], r13
	xor eax, eax
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_28
.LBB9_29:
	add rsp, 88
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB9_25:
	mov byte ptr [rbx + 1], 0
.LBB9_26:
	mov al, 1
	mov byte ptr [rbx], al
	cmp qword ptr [rsp + 8], 0
	je .LBB9_29
.LBB9_28:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_29
.LBB9_17:
	mov byte ptr [rbx + 1], al
	jmp .LBB9_26
	jmp .LBB9_23
.LBB9_23:
	mov rbx, rax
	cmp qword ptr [rsp + 8], 0
	jne .LBB9_20
.LBB9_24:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB9_20:
	lea rdi, [rsp + 8]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB9_24
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
