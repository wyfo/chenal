mpmc_recv:
	push r14
	push rbx
	sub rsp, 56
	mov rcx, qword ptr [rdi]
	lea rdi, [rcx + 128]
	mov qword ptr [rsp], rdi
	lea rbx, [rsp + 8]
	mov qword ptr [rsp + 8], 0
	mov rax, qword ptr [rcx + 256]
	mov r8, qword ptr [rcx + 560]
	and r8, rax
	mov rdx, qword ptr [rcx + 544]
	mov r9, r8
	shl r9, 4
	mov r10, qword ptr [rdx + r9 + 8]
	cmp r10, rax
	jne .LBB14_3
	add rdx, r9
	mov rdx, qword ptr [rdx]
	#MEMBARRIER
	mov r9, qword ptr [rcx + 552]
	dec r9
	cmp r8, r9
	jne .LBB14_7
	mov r8, qword ptr [rcx + 560]
	or r8, rax
	inc r8
	lock cmpxchg	qword ptr [rcx + 256], r8
	je .LBB14_8
.LBB14_3:
	mov rsi, qword ptr [rsi]
	mov rdx, rbx
	mov rcx, rax
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp rax, 2
	jne .LBB14_6
	mov eax, 2
	cmp qword ptr [rsp + 8], 0
	je .LBB14_13
	jmp .LBB14_14
.LBB14_6:
	and eax, 1
	cmp qword ptr [rsp + 8], 0
	jne .LBB14_14
.LBB14_13:
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB14_7:
	lea r8, [rax + 1]
	lock cmpxchg	qword ptr [rcx + 256], r8
	jne .LBB14_3
.LBB14_8:
	mov rax, qword ptr [rcx + 384]
	test al, 1
	je .LBB14_11
	add rcx, 384
	mov rdi, rcx
	mov r14, rdx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	mov rdx, r14
.LBB14_11:
	xor eax, eax
	cmp qword ptr [rsp + 8], 0
	je .LBB14_13
.LBB14_14:
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
	jne .LBB14_17
.LBB14_16:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB14_17:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB14_16
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
