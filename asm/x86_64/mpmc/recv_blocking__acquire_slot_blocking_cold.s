chenal::channel::Chan<T,Ch>::acquire_slot_blocking_cold:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push rbx
	sub rsp, 104
	mov qword ptr [rsp + 16], rcx
	mov r12, rdx
	mov r14, rsi
	mov qword ptr [rsp], rdi
	mov qword ptr [rsp + 24], 0
	xor ecx, ecx
	mov r15, qword ptr [rip + std::thread::functions::yield_now@GOTPCREL]
.LBB11_1:
	mov dword ptr [rsp + 12], ecx
	mov rax, r12
	jmp .LBB11_2
.LBB11_11:
	lea rcx, [r12 + 1]
	mov rax, r12
	lock cmpxchg	qword ptr [r14 + 128], rcx
	je .LBB11_13
.LBB11_2:
	mov r12, rax
	mov rbx, qword ptr [r14 + 432]
	and rbx, rax
	mov rax, qword ptr [r14 + 416]
	mov rcx, rbx
	shl rcx, 4
	lea r13, [rax + rcx]
	mov rax, qword ptr [rax + rcx + 8]
	cmp rax, r12
	jne .LBB11_3
.LBB11_9:
	mov r13, qword ptr [r13]
	#MEMBARRIER
	mov rax, qword ptr [r14 + 424]
	dec rax
	cmp rbx, rax
	jne .LBB11_11
	mov rcx, qword ptr [r14 + 432]
	or rcx, r12
	inc rcx
	mov rax, r12
	lock cmpxchg	qword ptr [r14 + 128], rcx
	jne .LBB11_2
	jmp .LBB11_13
.LBB11_3:
	mov rax, qword ptr [r14 + 128]
	cmp rax, r12
	jne .LBB11_2
	mov rax, qword ptr [r14]
	mov rcx, qword ptr [r14 + 432]
	shr rcx
	mov edx, -2
	sub edx, ecx
	and edx, eax
	cmp r12, rdx
	je .LBB11_27
	xor ebp, ebp
.LBB11_6:
	cmp ebp, 6
	ja .LBB11_16
	mov eax, 1
.LBB11_8:
	pause
	mov edx, eax
	mov ecx, ebp
	shr edx, cl
	inc eax
	test edx, edx
	je .LBB11_8
	jmp .LBB11_18
.LBB11_16:
	call r15
	cmp ebp, 11
	jae .LBB11_19
.LBB11_18:
	inc ebp
.LBB11_19:
	mov rax, qword ptr [r14 + 128]
	cmp rax, r12
	jne .LBB11_2
	mov rax, qword ptr [r13 + 8]
	cmp rax, r12
	jne .LBB11_6
	jmp .LBB11_9
.LBB11_27:
	mov eax, eax
	cmp r12, rax
	jne .LBB11_28
	test byte ptr [rsp + 12], 1
	je .LBB11_34
	mov rdi, qword ptr [rsp + 16]
	call qword ptr [rip + chenal::blocking::Parker::park@GOTPCREL]
	xor ecx, ecx
	cmp al, 2
	je .LBB11_1
	jmp .LBB11_40
.LBB11_34:
	cmp qword ptr [rsp + 24], 0
	jne .LBB11_36
	lea rax, [rsp + 32]
	xorps xmm0, xmm0
	movups xmmword ptr [rax], xmm0
	mov qword ptr [rax + 16], 0
	lea rax, [r14 + 304]
	mov qword ptr [rsp + 24], rax
	mov byte ptr [rsp + 64], 2
.LBB11_36:
	mov rax, qword ptr [rip + chenal::blocking::PARK_WAKER@GOTPCREL]
	mov qword ptr [rsp + 72], rax
	mov qword ptr [rsp + 80], rax
	mov qword ptr [rsp + 88], 0
	lea rdi, [rsp + 24]
	lea rsi, [rsp + 72]
	call aiq::wait_queue::Wait<Q,SP>::poll_wait
	mov ecx, eax
	jmp .LBB11_1
.LBB11_13:
	mov rax, qword ptr [r14 + 256]
	test al, 1
	je .LBB11_15
	add r14, 256
	mov rdi, r14
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
.LBB11_15:
	mov rcx, qword ptr [rsp]
	mov qword ptr [rcx + 8], r13
	xor eax, eax
	mov byte ptr [rcx], al
	cmp qword ptr [rsp + 24], 0
	jne .LBB11_31
.LBB11_32:
	add rsp, 104
	pop rbx
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp
	ret
.LBB11_28:
	mov rcx, qword ptr [rsp]
	mov byte ptr [rcx + 1], 0
	jmp .LBB11_29
.LBB11_40:
	mov rcx, qword ptr [rsp]
	mov byte ptr [rcx + 1], al
.LBB11_29:
	mov al, 1
	mov byte ptr [rcx], al
	cmp qword ptr [rsp + 24], 0
	je .LBB11_32
.LBB11_31:
	lea rdi, [rsp + 24]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB11_32
	jmp .LBB11_25
	jmp .LBB11_25
.LBB11_25:
	mov rbx, rax
	cmp qword ptr [rsp + 24], 0
	jne .LBB11_21
.LBB11_26:
	mov rdi, rbx
	call _Unwind_Resume@PLT
.LBB11_21:
	lea rdi, [rsp + 24]
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB11_26
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
