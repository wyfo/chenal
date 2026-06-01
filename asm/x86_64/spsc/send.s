spsc_send:
	push r14
	push rbx
	push rax
	mov rax, rdx
	mov r14, rsi
	mov rbx, qword ptr [rdi]
	mov rdx, qword ptr [rbx + 128]
	mov rcx, rdx
	shr rcx, 32
	cmp edx, ecx
	je .LBB3_10
	mov rcx, qword ptr [rbx + 544]
	test rcx, rcx
	jne .LBB3_10
.LBB3_2:
	mov rax, qword ptr [rbx + 520]
	mov rcx, qword ptr [rbx + 536]
	and rcx, rdx
	mov qword ptr [rax + 8*rcx], r14
	mov rax, qword ptr [rbx + 528]
	dec rax
	cmp rcx, rax
	jne .LBB3_4
	mov ecx, dword ptr [rbx + 536]
	or ecx, edx
	inc ecx
	movabs rax, -4294967296
	and rax, rdx
	or rax, rcx
	jmp .LBB3_5
.LBB3_4:
	lea rax, [rdx + 1]
.LBB3_5:
	xchg qword ptr [rbx + 128], rax
	mov rax, qword ptr [rbx + 544]
	test rax, rax
	jne .LBB3_15
.LBB3_6:
	mov rsi, qword ptr [rbx + 504]
	cmp rsi, 1
	jbe .LBB3_13
.LBB3_7:
	xor eax, eax
.LBB3_8:
.LBB3_9:
	mov rdx, r14
	add rsp, 8
	pop rbx
	pop r14
	ret
.LBB3_10:
	mov rsi, qword ptr [rax]
	lea rdi, [rbx + 128]
	call chenal::channel::Chan<T,Ch>::poll_acquire_slot_cold
	cmp rax, 2
	je .LBB3_8
	test rax, rax
	je .LBB3_2
.LBB3_12:
	mov eax, 1
	jmp .LBB3_9
.LBB3_13:
	add rbx, 472
	mov rdi, rbx
	call qword ptr [rip + spmc_waker::SpmcWaker<_,_>::wake_unsync_cold@GOTPCREL]
	jmp .LBB3_7
.LBB3_15:
	lea rdi, [rbx + 128]
	mov rsi, rdx
	call qword ptr [rip + <chenal::spsc::array::Array<_,C> as chenal::internal::Channel>::write_slot::handle_closed@GOTPCREL]
	test al, 1
	je .LBB3_6
	mov r14, rdx
	jmp .LBB3_12
