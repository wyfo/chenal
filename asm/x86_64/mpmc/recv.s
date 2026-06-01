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
	jne .LBB12_7
	add rdx, r9
	mov rdx, qword ptr [rdx]
	#MEMBARRIER
	mov r9, qword ptr [rcx + 552]
	dec r9
	cmp r8, r9
	jne .LBB12_3
	mov r8, qword ptr [rcx + 560]
	or r8, rax
	inc r8
	jmp .LBB12_4
.LBB12_3:
	lea r8, [rax + 1]
.LBB12_4:
	lock cmpxchg	qword ptr [rcx + 256], r8
	jne .LBB12_7
.LBB12_5:
	mov rdi, qword ptr [rsp]
	mov rax, qword ptr [rdi + 256]
	test al, 1
	je .LBB12_6
	add rdi, 256
	mov r14, rdx
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
	mov rdx, r14
.LBB12_6:
	xor eax, eax
	cmp qword ptr [rsp + 8], 0
	jne .LBB12_17
.LBB12_16:
	add rsp, 56
	pop rbx
	pop r14
	ret
.LBB12_7:
	mov rsi, qword ptr [rsi]
	mov rdx, rbx
	mov rcx, rax
	call chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	cmp rax, 2
	jne .LBB12_12
	mov eax, 2
	cmp qword ptr [rsp + 8], 0
	je .LBB12_16
	jmp .LBB12_17
.LBB12_12:
	test al, 1
	je .LBB12_5
	mov eax, 1
	cmp qword ptr [rsp + 8], 0
	je .LBB12_16
.LBB12_17:
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
	jne .LBB12_20
.LBB12_19:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB12_20:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB12_19
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
