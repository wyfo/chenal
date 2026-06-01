mpmc_send:
	push r14
	push rbx
	sub rsp, 104
	mov rax, rsi
	mov rdi, qword ptr [rdi]
	lea rsi, [rdi + 128]
	mov qword ptr [rsp + 24], rsi
	mov qword ptr [rsp + 8], 1
	mov qword ptr [rsp + 16], rax
	lea rbx, [rsp + 32]
	mov qword ptr [rsp + 32], 0
	mov r8, qword ptr [rdi + 128]
	mov ecx, r8d
	mov rax, r8
	shr rax, 32
	cmp rcx, rax
	je .LBB12_4
	mov r9, qword ptr [rdi + 560]
	and r9, r8
	mov rax, qword ptr [rdi + 552]
	dec rax
	cmp r9, rax
	jae .LBB12_4
	lea r10, [r8 + 1]
	mov rax, r8
	lock cmpxchg	qword ptr [rsi], r10
	jne .LBB12_3
	shl r9, 4
	add r9, qword ptr [rdi + 544]
	jmp .LBB12_8
.LBB12_3:
	mov r8, rax
.LBB12_4:
	mov rdx, qword ptr [rdx]
	lea rdi, [rsp + 80]
	mov rcx, rbx
	call chenal::channel::Chan<T,Ch,SP>::poll_acquire_slot_cold
	cmp byte ptr [rsp + 80], 0
	je .LBB12_6
	mov eax, 2
	cmp qword ptr [rsp + 32], 0
	je .LBB12_23
	jmp .LBB12_22
.LBB12_6:
	mov r9, qword ptr [rsp + 88]
	mov rcx, qword ptr [rsp + 96]
.LBB12_8:
	mov r14, qword ptr [rsp + 16]
	cmp byte ptr [rsp + 8], 0
	mov qword ptr [rsp + 8], 0
	je .LBB12_14
	test r9, r9
	je .LBB12_10
	mov rdi, qword ptr [rsp + 24]
	#MEMBARRIER
	mov qword ptr [r9], r14
	mov qword ptr [r9 + 8], rcx
	mov rax, qword ptr [rdi + 304]
	test al, 1
	je .LBB12_12
	add rdi, 304
	call qword ptr [rip + aiq::queue::Queue<T,S,SP>::is_empty_locked@GOTPCREL]
.LBB12_12:
	xor eax, eax
	cmp qword ptr [rsp + 32], 0
	jne .LBB12_22
.LBB12_23:
	mov rdx, r14
	add rsp, 104
	pop rbx
	pop r14
	ret
.LBB12_10:
	mov eax, 1
	cmp qword ptr [rsp + 32], 0
	je .LBB12_23
.LBB12_22:
	lea rdi, [rsp + 32]
	mov rbx, rax
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov rax, rbx
	jmp .LBB12_23
.LBB12_14:
	call qword ptr [rip + <chenal::channel::SendFuture<T,Ch,B,SP> as core::future::future::Future>::poll::polled_after_completion@GOTPCREL]
	ud2
	mov r14, rax
	cmp qword ptr [rsp + 32], 0
	jne .LBB12_17
.LBB12_18:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB12_17:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB12_18
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
