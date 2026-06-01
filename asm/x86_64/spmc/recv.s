spmc_recv:
	push r14
	push rbx
	sub rsp, 56
	mov rcx, qword ptr [rdi]
	lea rdi, [rcx + 128]
	mov qword ptr [rsp], rdi
	mov qword ptr [rsp + 8], 0
	mov rax, qword ptr [rcx + 256]
	mov rdx, rax
	shr rdx, 32
	cmp eax, edx
	jne .LBB12_1
.LBB12_6:
	lea rbx, [rsp + 8]
	mov rsi, qword ptr [rsi]
	mov rdx, rbx
	mov rcx, rax
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp rax, 2
	jne .LBB12_12
	mov eax, 2
	cmp qword ptr [rsp + 8], 0
	je .LBB12_15
	jmp .LBB12_14
.LBB12_1:
	mov r8, qword ptr [rcx + 544]
	and r8, rax
	mov rdx, qword ptr [rcx + 528]
	mov rdx, qword ptr [rdx + 8*r8]
	#MEMBARRIER
	mov r9, qword ptr [rcx + 536]
	dec r9
	cmp r8, r9
	jne .LBB12_3
	mov r9d, dword ptr [rcx + 544]
	or r9d, eax
	inc r9d
	movabs r8, -4294967296
	and r8, rax
	or r8, r9
	jmp .LBB12_4
.LBB12_3:
	lea r8, [rax + 1]
.LBB12_4:
	lock cmpxchg	qword ptr [rcx + 256], r8
	jne .LBB12_6
	xor eax, eax
	cmp qword ptr [rsp + 8], 0
	jne .LBB12_14
.LBB12_15:
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB12_12:
	and eax, 1
	cmp qword ptr [rsp + 8], 0
	je .LBB12_15
.LBB12_14:
	lea rdi, [rsp + 8]
	mov rbx, rdx
	mov r14, rax
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov rax, r14
	mov rdx, rbx
	add rsp, 56
	pop rbx
	pop r14
	ret
	mov r14, rax
	cmp qword ptr [rsp + 8], 0
	jne .LBB12_10
.LBB12_11:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB12_10:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB12_11
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
