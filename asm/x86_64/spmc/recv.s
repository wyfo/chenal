spmc_recv:
	push r14
	push rbx
	sub rsp, 56
	mov rcx, qword ptr [rdi]
	lea rdi, [rcx + 128]
	mov qword ptr [rsp], rdi
	lea rbx, [rsp + 8]
	mov qword ptr [rsp + 8], 0
	mov rax, qword ptr [rcx + 256]
	mov rdx, rax
	shr rdx, 32
	cmp eax, edx
	jne .LBB13_4
.LBB13_1:
	mov rsi, qword ptr [rsi]
	mov rdx, rbx
	mov rcx, rax
	call chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	cmp rax, 2
	jne .LBB13_12
	mov eax, 2
	jmp .LBB13_14
.LBB13_4:
	mov r8, qword ptr [rcx + 544]
	and r8, rax
	mov rdx, qword ptr [rcx + 528]
	mov rdx, qword ptr [rdx + 8*r8]
	#MEMBARRIER
	mov r9, qword ptr [rcx + 536]
	dec r9
	cmp r8, r9
	jne .LBB13_6
	mov r9d, dword ptr [rcx + 544]
	or r9d, eax
	inc r9d
	movabs r8, -4294967296
	and r8, rax
	or r8, r9
	jmp .LBB13_7
.LBB13_6:
	lea r8, [rax + 1]
.LBB13_7:
	lock cmpxchg	qword ptr [rcx + 256], r8
	jne .LBB13_1
.LBB13_8:
	mov rdi, qword ptr [rsp]
	mov rsi, qword ptr [rdi + 384]
	cmp rsi, 1
	ja .LBB13_9
	add rdi, 352
	mov r14, rdx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	mov rdx, r14
.LBB13_9:
	xor eax, eax
	cmp qword ptr [rsp + 8], 0
	jne .LBB13_17
.LBB13_16:
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB13_12:
	test al, 1
	je .LBB13_8
	mov eax, 1
.LBB13_14:
	cmp qword ptr [rsp + 8], 0
	je .LBB13_16
.LBB13_17:
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
	jne .LBB13_20
.LBB13_19:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB13_20:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB13_19
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
