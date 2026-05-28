mpsc_send:
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
	je .LBB10_4
	mov r9, qword ptr [rdi + 544]
	and r9, r8
	mov rax, qword ptr [rdi + 536]
	dec rax
	cmp r9, rax
	jae .LBB10_4
	lea r10, [r8 + 1]
	mov rax, r8
	lock cmpxchg	qword ptr [rsi], r10
	jne .LBB10_3
	shl r9, 4
	add r9, qword ptr [rdi + 528]
	jmp .LBB10_8
.LBB10_3:
	mov r8, rax
.LBB10_4:
	mov rdx, qword ptr [rdx]
	lea rdi, [rsp + 80]
	mov rcx, rbx
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp byte ptr [rsp + 80], 0
	je .LBB10_6
	mov eax, 2
	cmp qword ptr [rsp + 32], 0
	je .LBB10_23
	jmp .LBB10_22
.LBB10_6:
	mov r9, qword ptr [rsp + 88]
	mov rcx, qword ptr [rsp + 96]
.LBB10_8:
	mov r14, qword ptr [rsp + 16]
	cmp byte ptr [rsp + 8], 0
	mov qword ptr [rsp + 8], 0
	je .LBB10_14
	test r9, r9
	je .LBB10_10
	mov rdi, qword ptr [rsp + 24]
	mov qword ptr [r9], r14
	mov qword ptr [r9 + 8], rcx
	mov rsi, qword ptr [rdi + 384]
	cmp rsi, 1
	ja .LBB10_12
	add rdi, 352
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
.LBB10_12:
	xor eax, eax
	cmp qword ptr [rsp + 32], 0
	jne .LBB10_22
.LBB10_23:
	mov rdx, r14
	add rsp, 104
	pop rbx
	pop r14
	ret
.LBB10_10:
	mov eax, 1
	cmp qword ptr [rsp + 32], 0
	je .LBB10_23
.LBB10_22:
	lea rdi, [rsp + 32]
	mov rbx, rax
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	mov rax, rbx
	jmp .LBB10_23
.LBB10_14:
	call qword ptr [rip + <chenal::channel::SendFuture<T,Ch,B> as core::future::future::Future>::poll::polled_after_completion@GOTPCREL]
	ud2
	mov r14, rax
	cmp qword ptr [rsp + 32], 0
	jne .LBB10_17
.LBB10_18:
	mov rdi, r14
	call _Unwind_Resume@PLT
.LBB10_17:
	mov rdi, rbx
	call <chenal::waiter::OptionCold<T> as core::ops::drop::Drop>::drop::drop_cold
	jmp .LBB10_18
	call qword ptr [rip + core::panicking::panic_in_cleanup@GOTPCREL]
